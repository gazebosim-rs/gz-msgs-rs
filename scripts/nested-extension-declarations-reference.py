#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Observe source-aware nested extension declarations without changing runtime admission."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[1]

def inputs():
    from google.protobuf import descriptor_pb2 as d
    rows=[]
    def add(kind=5,label=1,sequence='ABA',layout='ordinary',mode='ordinary'):
        files=d.FileDescriptorSet()
        f=files.file.add(name='extension_declarations.proto',package='probe',syntax='proto2')
        for name in ['A','B']:
            base=f.message_type.add(name=name)
            base.extension_range.add(start=100,end=200)
        e=f.enum_type.add(name='Choice');e.value.add(name='ZERO',number=0)
        root=f.message_type.add(name='Root')
        root.field.add(name='value',number=1,label=1,type=5,json_name='value')
        owner=root.nested_type.add(name='Child') if layout=='nested' else root
        if layout=='sections':
            owner.extension_range.add(start=300,end=310)
            owner.reserved_range.add(start=400,end=410)
            owner.reserved_name.append('old')
        for i,target in enumerate(sequence):
            v=owner.extension.add(name='extra_'+str(i),number=100+i,label=label,type=kind,extendee='.probe.'+target)
            if kind==11:v.type_name='.probe.A'
            elif kind==14:v.type_name='.probe.Choice'
            if mode=='explicit-json':v.json_name='extra'+str(i)
            elif mode=='custom-json':v.json_name='custom'+str(i)
            elif mode=='default':v.default_value='3'
            elif mode=='empty-options':v.options.SetInParent()
            elif mode=='deprecated':v.options.deprecated=False
            elif mode=='unknown-option':v.options.MergeFromString(bytes([0x80,0x7d,7]))
            elif mode=='oneof':
                owner.oneof_decl.add(name='selection');v.oneof_index=0
            elif mode=='proto3-optional':v.proto3_optional=True
            elif mode in ['space-name','newline-name','unicode-name','digit-name','empty-name']:
                v.name={'space-name':'BAD NAME','newline-name':'BAD\n','unicode-name':'☃','digit-name':'1BAD','empty-name':''}[mode]
                if v.name:v.name+=str(i)
            elif mode=='group':
                v.type=10;v.name='group'+str(i);v.type_name='.probe.Root.Group'+str(i)
                owner.nested_type.add(name='Group'+str(i))
            elif mode=='map':
                v.type=11;v.label=3;v.type_name='.probe.Root.Entry'+str(i)
                entry=owner.nested_type.add(name='Entry'+str(i));entry.options.map_entry=True
                entry.field.add(name='key',number=1,label=1,type=9)
                entry.field.add(name='value',number=2,label=1,type=5)
            elif mode=='message-set':
                v.type=11;v.type_name='.probe.B';f.message_type[0 if target=='A' else 1].options.message_set_wire_format=True
            elif mode=='custom-option':
                from google.protobuf.internal.encoder import _VarintBytes
                f.dependency.append('google/protobuf/descriptor.proto') if not f.dependency else None
                if not f.extension:f.extension.add(name='custom_field',number=50000,label=1,type=8,extendee='.google.protobuf.FieldOptions')
                v.options.MergeFromString(_VarintBytes(50000<<3)+b'\x01')
        if mode=='custom-option':
            historical=d.FileDescriptorSet.FromString((ROOT/'tests/fixtures/definitions/unsupported.desc').read_bytes())
            standard=next(v for v in historical.file if v.name=='google/protobuf/descriptor.proto')
            files.file.insert(0,standard)
        if mode=='proto3':f.syntax='proto3' 
        rows.append(dict(id=len(rows),kind=kind,label=label,sequence=sequence,layout=layout,mode=mode,descriptor_hex=files.SerializeToString().hex()))
    for kind,label,layout in itertools.product([1,2,3,4,5,6,7,8,9,11,12,13,14,15,16,17,18],[1,3],['ordinary','nested','sections']):
        add(kind,label,layout=layout)
    for sequence in ['A','AB','AA','BAAB']:
        add(sequence=sequence)
    for mode in ['explicit-json','custom-json','default','empty-options','deprecated','unknown-option','oneof','proto3-optional','proto3','group','map','message-set','custom-option']:
        add(mode=mode)
    add(label=2)
    for mode in ['space-name','newline-name','unicode-name','digit-name','empty-name']:
        add(sequence='A',mode=mode)
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--upstream',required=True,type=Path);parser.add_argument('--write-fixture',action='store_true');args=parser.parse_args()
    out=ROOT/'artifacts/nested-extension-declarations-reference';out.mkdir(parents=True,exist_ok=True)
    baseline=json.loads((ROOT/'tests/fixtures/definitions/cli.json').read_text())
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    up=args.upstream.resolve();binary=up/'build/bin/gz-msgs'
    assert subprocess.check_output(['git','-C',str(up),'rev-parse','HEAD'],text=True).strip()==baseline['upstream_commit']
    assert sha(binary)==baseline['native_binary_sha256']
    assert sha(up/'core/cmd/msgs_main.cc')==baseline['native_source_sha256']
    assert sha(ROOT/'tests/definition_peer.cc')==baseline['peer_source_sha256']
    assert sha(ROOT/'tests/fixtures/definitions/unsupported.desc')==baseline['inputs']['tests/fixtures/definitions/unsupported.desc']
    assert sha(ROOT/'schemas/manifest.json')==baseline['schema_manifest_sha256']
    assert subprocess.check_output(['pkg-config','--modversion','protobuf'],text=True).strip()=='3.21.12'
    libraries={}
    for line in subprocess.check_output(['ldd',str(binary)],text=True).splitlines():
        words=line.split()
        if len(words)>2 and words[1]=='=>' and words[0].startswith(('libgz-','libprotobuf')):libraries[words[0]]=sha(Path(words[2]))
    assert libraries==baseline['native_libraries_sha256']
    peer=out/'peer';subprocess.run(['c++','-std=c++17','-Wall','-Wextra','-Werror',str(ROOT/'tests/definition_peer.cc'),'-lprotobuf','-o',str(peer)],check=True)
    for line in subprocess.check_output(['ldd',str(peer)],text=True).splitlines():
        words=line.split()
        if len(words)>2 and words[1]=='=>' and words[0].startswith('libprotobuf'):assert sha(Path(words[2]))==libraries[words[0]]
    fixture=ROOT/'tests/fixtures/nested-extension-declarations.json'
    rows=inputs();old=None if args.write_fixture else json.loads(fixture.read_text())
    if old:assert [{k:v for k,v in r.items() if k not in ['native','native_cli']} for r in old['rows']]==rows
    def observation(p):return dict(exit=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode())
    for row in rows:
        p=out/'input.desc';p.write_bytes(bytes.fromhex(row['descriptor_hex']))
        row['native']=observation(subprocess.run([str(peer),str(p)],capture_output=True,timeout=10))
        row['native_cli']=observation(subprocess.run(['gz-msgs','--info','probe.Root'],executable=binary,env=dict(os.environ,GZ_DESCRIPTOR_PATH=str(p.relative_to(ROOT))),cwd=ROOT,capture_output=True,timeout=10))
    result=dict(reference_script_sha256=sha(Path(__file__)), schema_manifest_sha256=baseline['schema_manifest_sha256'], standard_descriptor_fixture_sha256=baseline['inputs']['tests/fixtures/definitions/unsupported.desc'], protobuf='3.21.12',upstream_commit=baseline['upstream_commit'],native_binary_sha256=sha(binary),native_source_sha256=baseline['native_source_sha256'],native_libraries_sha256=libraries,peer_source_sha256=sha(ROOT/'tests/definition_peer.cc'),rows=rows)
    if args.write_fixture:fixture.write_text(json.dumps(result,indent=2)+'\n')
    else:assert result==old
    print('complete observations',len(rows),'native admitted',sum(r['native']['exit']==0 for r in rows))
if __name__=='__main__':main()
