#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture complete canonical native --info output, including synthetic messages."""
import hashlib, json, pathlib, subprocess
from google.protobuf import descriptor_pb2, descriptor_pool
root=pathlib.Path(__file__).resolve().parents[1]
reference=root.parent/'gz-msgs'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
subprocess.run(['python3',str(root/'scripts/native-peer.py')],check=True)
files=descriptor_pb2.FileDescriptorSet.FromString((root/'target/native/messages.pb').read_bytes())
pool=descriptor_pool.DescriptorPool()
for file in files.file:pool.AddSerializedFile(file.SerializeToString())
names=[]
def visit(message):
    names.append(message.full_name)
    for child in message.nested_types:visit(child)
for file in files.file:
    for message in pool.FindFileByName(file.name).message_types_by_name.values():visit(message)
records={}
binary=reference/'build/bin/gz-msgs'
for name in sorted(names):
    result=subprocess.run([str(binary),'--info',name],check=True,capture_output=True,text=True)
    assert not result.stderr,(name,result.stderr)
    file=pool.FindMessageTypeByName(name).file.name
    prefix=f'Name: {name}\nFile: {file}\n\n'
    assert result.stdout.startswith(prefix) and result.stdout.endswith('\n')
    records[name]={'definition':result.stdout[len(prefix):-1],'output':result.stdout}
combined=subprocess.check_output([str(binary),'--info',*records],text=True)
assert combined==''.join(record['output'] for record in records.values())
corpus=root/'schemas/cli-info.json'
corpus.write_text(json.dumps(records,indent=2)+'\n')
manifest={'upstream_commit':'89c0c3d1474f545cb889335daf930bfdab6ab852','native_binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'schema_manifest_sha256':hashlib.sha256((root/'schemas/manifest.json').read_bytes()).hexdigest(),'corpus_sha256':hashlib.sha256(corpus.read_bytes()).hexdigest(),'protobuf_version':subprocess.check_output(['pkg-config','--modversion','protobuf'],text=True).strip(),'messages':len(records),'generation':'Complete stdout of pinned native gz-msgs --info for all canonical/imported/nested/synthetic message descriptors; definition is native Descriptor::DebugString without CLI headings/trailing separator.'}
(root/'schemas/cli-info.manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(len(records),'complete native info outputs captured')
