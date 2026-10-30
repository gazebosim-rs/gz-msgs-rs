#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Characterize declared enum extension options with unchanged descriptor admission."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[1]

def inputs():
    from google.protobuf import descriptor_pb2 as d, descriptor_pool, message_factory
    import importlib.util
    spec=importlib.util.spec_from_file_location('nested',ROOT/'scripts/nested-extension-declarations-reference.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    previous=module.inputs()
    # Maintained integer serialization preserves undeclared proto2 enum numbers.
    file=d.FileDescriptorProto(name='surrogate.proto',syntax='proto2')
    msg=file.message_type.add(name='Options')
    for name in ['ctype','jstype']:
        msg.field.add(name=name,number=d.FieldOptions.DESCRIPTOR.fields_by_name[name].number,label=1,type=5)
    pool=descriptor_pool.DescriptorPool();pool.Add(file)
    integers=message_factory.MessageFactory(pool).GetPrototype(pool.FindMessageTypeByName('Options'))
    rows=[]
    def add(base,profiles,mode):
        files=d.FileDescriptorSet.FromString(bytes.fromhex(base['descriptor_hex']))
        root=files.file[-1].message_type[2]
        owner=root.nested_type[0] if base['layout']=='nested' else root
        profiles=profiles[:len(owner.extension)]
        for extension,profile in zip(owner.extension,profiles):
            for name,value in profile.items():
                if value is None:continue
                if name in ['ctype','jstype']:extension.options.MergeFromString(integers(**{name:value}).SerializeToString())
                else:setattr(extension.options,name,value)
        row={k:v for k,v in base.items() if k!='descriptor_hex'}
        row.update(id=len(rows),mode=mode,profiles=profiles,descriptor_hex=files.SerializeToString().hex());rows.append(row)
    bases=[v for v in previous if v['mode']=='ordinary' and v['label'] in [1,3] and v['sequence']=='ABA']
    for base in bases:
        for name,value in itertools.product(['ctype','jstype'],[None,0,1,2,-1,3,2147483647,-2147483648]):
            add(base,[{name:value}]*3,'matrix')
        for values in [(1,0,False,True),(2,1,None,False),(0,2,True,False),(None,None,True,True),(None,0,False,False),(0,None,None,True)]:
            profile=dict(zip(['ctype','jstype','packed','deprecated'],values))
            add(base,[profile,{},profile],'mixed')
    for mode in ['explicit-json','custom-json','default','unknown-option','proto3-optional','proto3','group','map','message-set','custom-option','space-name','newline-name','unicode-name','empty-name']:
        base=next(v for v in previous if v['mode']==mode)
        add(base,[{'ctype':0,'jstype':0}]*3,mode)
    base=bases[0]
    add(base,[{'ctype':0,'lazy':False}]*3,'unhandled-option')
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--upstream',required=True,type=Path);parser.add_argument('--write-fixture',action='store_true');args=parser.parse_args()
    out=ROOT/'artifacts/extension-enum-options-reference';out.mkdir(parents=True,exist_ok=True)
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
    fixture=ROOT/'tests/fixtures/extension-enum-options.json'
    rows=inputs();old=None if args.write_fixture else json.loads(fixture.read_text())
    if old:assert [{k:v for k,v in r.items() if k not in ['native','native_cli']} for r in old['rows']]==rows
    def observation(p):return dict(exit=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode())
    for row in rows:
        p=out/'input.desc';p.write_bytes(bytes.fromhex(row['descriptor_hex']))
        row['native']=observation(subprocess.run([str(peer),str(p)],capture_output=True,timeout=10))
        row['native_cli']=observation(subprocess.run(['gz-msgs','--info','probe.Root'],executable=binary,env=dict(os.environ,GZ_DESCRIPTOR_PATH=str(p.relative_to(ROOT))),cwd=ROOT,capture_output=True,timeout=10))
    result=dict(packed_corpus_sha256=sha(ROOT/'tests/fixtures/extension-packed.json'), packed_reference_script_sha256=sha(ROOT/'scripts/extension-packed-reference.py'), nested_corpus_sha256=sha(ROOT/'tests/fixtures/nested-extension-declarations.json'), nested_reference_script_sha256=sha(ROOT/'scripts/nested-extension-declarations-reference.py'), deprecated_corpus_sha256=sha(ROOT/'tests/fixtures/extension-deprecated.json'), deprecated_reference_script_sha256=sha(ROOT/'scripts/extension-deprecated-reference.py'), reference_script_sha256=sha(Path(__file__)), schema_manifest_sha256=baseline['schema_manifest_sha256'], standard_descriptor_fixture_sha256=baseline['inputs']['tests/fixtures/definitions/unsupported.desc'], protobuf='3.21.12',upstream_commit=baseline['upstream_commit'],native_binary_sha256=sha(binary),native_source_sha256=baseline['native_source_sha256'],native_libraries_sha256=libraries,peer_source_sha256=sha(ROOT/'tests/definition_peer.cc'),rows=rows)
    if args.write_fixture:fixture.write_text(json.dumps(result,indent=2)+'\n')
    else:assert result==old
    print('complete observations',len(rows),'native admitted',sum(r['native']['exit']==0 for r in rows))
if __name__=='__main__':main()
