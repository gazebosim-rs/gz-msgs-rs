#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Replay native file-cache workflows; bind only the explicit temporary input directory."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import shlex
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from google.protobuf import descriptor_pb2 as d
ROOT = Path(__file__).resolve().parents[1]
def inputs():
    def file(name, message):
        value=d.FileDescriptorProto(name=name,package='audit',syntax='proto2')
        value.message_type.add(name=message)
        return value
    a=file('a.proto','A');b=file('b.proto','B');dep=file('dep.proto','Dep')
    partial=file('a.proto','A');partial.dependency.append('dep.proto')
    partial.message_type[0].field.add(name='dep',number=1,label=1,type=11,type_name='.audit.Dep')
    result={}
    for name, files in [('A',[a]),('B',[b]),('Dep',[dep]),('partial',[b,partial]),('empty',[])]:
        value=d.FileDescriptorSet();value.file.extend(files);result[name]=value.SerializeToString()
    return result
def cases():
    rows=[('replace',p,q) for p,q in [('relative','dot'),('dot','parent'),('parent','absolute'),('absolute','absolute-parent'),('alias','relative'),('relative','alias')]]
    rows += [('delete','relative',q) for q in ['relative','dot','parent','absolute','absolute-dot','absolute-parent','alias','absolute-alias']]
    rows += [(action,'relative',q) for action in ['replace','delete'] for q in ['trailing','absolute-trailing','trailing-dot','notdir','missing-parent','absolute-missing-parent']]
    rows += [('retarget','alias','alias'),('missing','relative','dot'),('dangling','alias','alias'),('malformed','relative','absolute'),('partial','relative','dot'),('empty','relative','absolute')]
    rows += [('delete-parent','nested',q) for q in ['nested','nested-dot','nested-absolute','nested-absolute-parent']]
    rows += [('dangling-directory','directory-alias',q) for q in ['directory-alias','directory-alias-absolute']]
    rows += [('symlink-parent','directory-alias-parent','directory-alias-parent'),('loop','relative','alias'),('cached-loop','relative','absolute')]
    rows += [('raw-replace','raw','raw'),('raw-delete','raw','raw'),('raw-delete','raw','raw-absolute'),('raw-distinct','raw','raw-other'),('raw-distinct','raw','replacement')]
    return rows
def bind_input(data, directory):
    """Literal bytes separated by occurrences of this explicitly supplied input."""
    parts=data.split(os.fsencode(directory));result=[]
    for i,part in enumerate(parts):
        if i:result.append({'case_directory':True})
        result.append({'hex':part.hex()})
    return result
def expand(parts,directory):
    return b''.join(os.fsencode(directory) if p.get('case_directory') else bytes.fromhex(p['hex']) for p in parts)
def no_core_dump():
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--upstream',required=True,type=Path);parser.add_argument('--write-fixture',action='store_true');args=parser.parse_args()
    sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
    baseline=json.loads((ROOT/'tests/fixtures/definitions/cli.json').read_text());up=args.upstream.resolve()
    assert subprocess.check_output(['git','-C',str(up),'rev-parse','HEAD'],text=True).strip()==baseline['upstream_commit']
    subprocess.run(['git','-C',str(up),'diff','--quiet'],check=True)
    source_paths=['core/src/DynamicFactory.cc','core/src/MessageFactory.cc','core/include/gz/msgs/MessageFactory.hh','test/integration/factory_descriptor_retry.cc','test/integration/Factory_TEST.cc','test/integration/factory_lazy.cc','test/integration/factory_concurrency.cc']
    source={p:sha(up/p) for p in source_paths}
    include=Path(subprocess.check_output(['pkg-config','--variable=includedir','gz-msgs'],text=True).strip())
    assert sha(include/'gz/msgs/MessageFactory.hh')==source['core/include/gz/msgs/MessageFactory.hh']
    output=ROOT/'artifacts/factory-cache-reference';output.mkdir(parents=True,exist_ok=True)
    values=inputs();data_dir=output/'inputs';data_dir.mkdir(exist_ok=True)
    for name,data in values.items():(data_dir/(name+'.desc')).write_bytes(data)
    peer=output/'factory-cache-reference';flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs'],text=True))
    lib=Path(subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()).resolve()
    subprocess.run(['c++','-std=c++17','-I'+str(up/'core/include'),str(ROOT/'tests/factory_cache_reference.cc'),*flags,'-Wl,--disable-new-dtags','-Wl,-rpath,'+str(lib),'-o',str(peer)],check=True)
    env=dict(os.environ,LD_LIBRARY_PATH=str(lib));env.pop('GZ_DESCRIPTOR_PATH',None);env.pop('GZ_MSGS_INSTALL_PREFIX',None)
    libraries={}
    for line in subprocess.check_output(['ldd',str(peer)],text=True,env=env).splitlines():
        words=line.split()
        if len(words)>2 and words[1]=='=>' and Path(words[2]).is_file():libraries[words[0]]=sha(Path(words[2]))
    assert {n:h for n,h in libraries.items() if n.startswith(('libgz-','libprotobuf'))}==baseline['native_libraries_sha256']
    fixture=ROOT/'tests/fixtures/factory-cache.json';old=None if args.write_fixture else json.loads(fixture.read_text())
    rows=[];raw=[]
    for index,(action,first,second) in enumerate(cases()):
        with tempfile.TemporaryDirectory(prefix='case-',dir=output) as directory:
            value=subprocess.run([str(peer),action,first,second,str(data_dir)],cwd=directory,env=env,capture_output=True,timeout=10,preexec_fn=no_core_dump)
            row={'id':f'{action}:{first}:{second}','action':action,'first':first,'second':second,'native':{'exit':value.returncode,'stdout':bind_input(value.stdout,directory),'stderr':bind_input(value.stderr,directory)}}
            row['states']=[[int(v) for v in line.split()[1:]] for line in value.stdout.splitlines() if line.startswith(b'STATE ')]
            raw.append({'id':row['id'],'case_directory':directory,'exit':value.returncode,'stdout_hex':value.stdout.hex(),'stderr_hex':value.stderr.hex()})
            if old:
                expected=old['rows'][index]
                assert row['id']==expected['id']
                assert value.returncode==expected['native']['exit'],row['id']
                assert value.stdout==expand(expected['native']['stdout'],directory),row['id']
                assert value.stderr==expand(expected['native']['stderr'],directory),row['id']
            rows.append(row)
    original_baseline=json.loads((ROOT/'tests/fixtures/factory-state-reference.json').read_text())
    pinned={r['case']:r['binary_sha256'] for r in original_baseline['original_native_tests']}
    original_specs=[('INTEGRATION_factory_descriptor_retry','FactoryDescriptorRetryTest.ReloadAfterMissingImport'),('INTEGRATION_factory_lazy','FactoryLazyTest.DescriptorPathReadAtFirstDynamicUse'),('INTEGRATION_factory_concurrency','FactoryConcurrency.ParallelUse'),('INTEGRATION_Factory_TEST','FactoryTest.DynamicMessageWithEnv'),('INTEGRATION_Factory_TEST','FactoryTest.MultipleMessagesInAProto')]
    originals=[]
    for binary,case in original_specs:
        executable=up/'build/bin'/binary
        if case in pinned:assert sha(executable)==pinned[case],case
        report=output/(case+'.xml')
        value=subprocess.run([str(executable),'--gtest_filter='+case,'--gtest_output=xml:'+str(report)],cwd=ROOT,env=env,capture_output=True,timeout=30)
        (output/(case+'.log')).write_bytes(value.stdout+value.stderr)
        assert value.returncode==0,case
        tests=[suite.attrib['name']+'.'+t.attrib['name'] for suite in ET.parse(report).getroot() for t in suite.findall('testcase') if t.attrib.get('status')=='run' and not list(t)]
        assert tests==[case],case
        actual={}
        for line in subprocess.check_output(['ldd',str(executable)],text=True,env=env).splitlines():
            words=line.split()
            if len(words)>2 and words[1]=='=>' and Path(words[2]).is_file():actual[words[0]]=sha(Path(words[2]))
        originals.append({'case':case,'binary_sha256':sha(executable),'libraries_sha256':actual})
    result={'upstream_commit':baseline['upstream_commit'],'native_sources_sha256':source,'schema_manifest_sha256':sha(ROOT/'schemas/manifest.json'),'native_libraries_sha256':libraries,'peer_source_sha256':sha(ROOT/'tests/factory_cache_reference.cc'),'reference_script_sha256':sha(Path(__file__)),'inputs':{n:v.hex() for n,v in values.items()},'original_native_tests':originals,'rows':rows}
    if args.write_fixture:fixture.write_text(json.dumps(result,indent=2)+'\n')
    else:assert result==old
    (output/'actual-observations.json').write_text(json.dumps({'peer_sha256':sha(peer),'compiler':subprocess.check_output(['c++','--version'],text=True),'observations':raw},indent=2)+'\n')
    print('native cache workflows',len(rows),'normal completions',sum(r['native']['exit']==0 for r in rows),'explicit native aborts',sum(r['native']['exit']<0 for r in rows))
if __name__=='__main__':main()
