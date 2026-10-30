#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Record complete byte-oriented native CLI observations on the pinned Linux reference."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]

def inputs():
    rows=[]
    def add(label,args,environment=False):
        native=list(args)
        if native and native[0] in [b'info',b'list']:native[0]=b'--'+native[0]
        rows.append(dict(label=label,args_hex=[v.hex() for v in args],native_args_hex=[v.hex() for v in native],external_path=environment))
    for label,value in [('ff',b'\xff'),('continuation',b'\x80'),('middle',b'a\xffb'),('truncated',b'\xe2\x82'),('overlong',b'\xc0\xaf'),('unicode','☃'.encode())]:
        add(label+'-separate',[b'--info',value]);add(label+'-long-attached',[b'--info='+value]);add(label+'-short-attached',[b'-i'+value])
    for label,args in [
        ('bad-then-valid',[b'--info',b'\xff',b'gz.msgs.Time']),
        ('valid-then-bad',[b'--info',b'gz.msgs.Time',b'\xff']),
        ('repeated-info',[b'-i',b'\xff',b'--info=gz.msgs.Time']),
        ('word-alias',[b'info',b'\xff',b'gz.msgs.Time']),
        ('nested-vector',[b'--info=[\xff,[gz.msgs.Time,a\x80b]]']),
        ('empty-vector-required',[b'--info=[]',b'\xff']),
        ('negative-name',[b'--info',b'-1\xff']),
        ('unbracketed-comma',[b'--info',b'\xff,gz.msgs.Time']),
        ('escape-extra',[b'--',b'\xff']),
        ('escape-after-name',[b'--info',b'gz.msgs.Time',b'--',b'\xff']),
        ('required-escape',[b'--info',b'--',b'\xff']),
        ('positional',[b'\xff']),('unknown-long',[b'--\xff']),('unknown-short',[b'-\xff']),
        ('known-short-then-unknown',[b'-l\xff']),('unknown-before-known',[b'-\xffh']),
        ('help-extra',[b'--help',b'\xff']),('version-extra',[b'--version',b'\xff']),
        ('help-short-unknown',[b'-h\xff']),('version-short-unknown',[b'-v\xff']),
        ('help-attached',[b'--help=\xff']),('list-conflict',[b'--list',b'--info',b'\xff']),
        ('help-conflict',[b'--help',b'--list',b'--info',b'\xff']),
        ('bad-list-before-help',[b'--list=\xff',b'--help']),
        ('bad-version-before-help',[b'--version=\xff',b'--help']),
        ('two-extras',[b'\xff',b'--\x80']),('bad-name-plus-extra',[b'--info=\xff',b'--\x80']),
    ]:add(label,args)
    for name in [b'list',b'version']:
        for label,value in [('invalid',b'\xff'),('suffix',b'1\xff'),('overflow',b'18446744073709551616\xff'),('negative-overflow',b'-18446744073709551616\xff'),('space-overflow',b' -18446744073709551616\xff')]:
            add(name.decode()+'-'+label,[b'--'+name+b'='+value])
    add('raw-descriptor-path',[b'--info',b'byteargv.External'],True)
    return rows

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--upstream',required=True,type=Path);parser.add_argument('--write-fixture',action='store_true');args=parser.parse_args()
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();baseline=json.loads((ROOT/'tests/fixtures/definitions/cli.json').read_text());up=args.upstream.resolve();binary=up/'build/bin/gz-msgs'
    assert subprocess.check_output(['git','-C',str(up),'rev-parse','HEAD'],text=True).strip()==baseline['upstream_commit']
    assert sha(binary)==baseline['native_binary_sha256'];assert sha(up/'core/cmd/msgs_main.cc')==baseline['native_source_sha256']
    libraries={}
    for line in subprocess.check_output(['ldd',str(binary)],text=True).splitlines():
        w=line.split()
        if len(w)>2 and w[1]=='=>' and w[0].startswith(('libgz-','libprotobuf')):libraries[w[0]]=sha(Path(w[2]))
    assert libraries==baseline['native_libraries_sha256']
    output=ROOT/'artifacts/os-argv-reference';output.mkdir(parents=True,exist_ok=True)
    from google.protobuf import descriptor_pb2 as d
    files=d.FileDescriptorSet();file=files.file.add(name='byte_argv_external.proto',package='byteargv',syntax='proto3');file.message_type.add(name='External')
    path=os.fsencode(output)+b'/external-\xff.desc'
    with open(path,'wb') as stream:stream.write(files.SerializeToString())
    env=dict(os.environb);env.pop(b'GZ_DESCRIPTOR_PATH',None);env.pop(b'GZ_MSGS_INSTALL_PREFIX',None)
    def observe(values,external=False):
        child=dict(env)
        if external:child[b'GZ_DESCRIPTOR_PATH']=os.path.relpath(path,os.fsencode(ROOT))
        x=subprocess.run([b'gz-msgs',*values],executable=os.fsencode(binary),cwd=ROOT,env=child,capture_output=True,timeout=10)
        return dict(exit=x.returncode,stdout_hex=x.stdout.hex(),stderr_hex=x.stderr.hex())
    controls={name:observe([b'--'+name.encode()]) for name in ['help','version','list']}
    rows=inputs()
    for row in rows:
        row['native']=observe([bytes.fromhex(v) for v in row['native_args_hex']],row['external_path'])
        row['binding']='exact'
        for name,value in controls.items():
            if row['native']==value:row['binding']=name;break
    result=dict(upstream_commit=baseline['upstream_commit'],native_binary_sha256=sha(binary),native_source_sha256=baseline['native_source_sha256'],native_libraries_sha256=libraries,reference_script_sha256=sha(Path(__file__)),schema_manifest_sha256=sha(ROOT/'schemas/manifest.json'),old_cli_sha256={name:sha(ROOT/'tests/fixtures'/name) for name in ['cli-options.json','cli-flag-values.json','cli-info-vectors.json']},external_descriptor_hex=files.SerializeToString().hex(),controls=controls,rows=rows)
    fixture=ROOT/'tests/fixtures/os-argv.json'
    if args.write_fixture:fixture.write_text(json.dumps(result,indent=2)+'\n')
    else:assert result==json.loads(fixture.read_text())
    print('complete native byte-argv observations',len(rows))
if __name__=='__main__':main()
