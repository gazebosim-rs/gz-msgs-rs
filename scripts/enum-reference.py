#!/usr/bin/env python3
"""Capture native enum conversion policy against canonical enum descriptors."""
import hashlib, json, pathlib, shlex, subprocess
from google.protobuf import descriptor_pb2, descriptor_pool
root=pathlib.Path(__file__).resolve().parents[1]
reference=root.parent/'gz-msgs'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()=='89c0c3d1474f545cb889335daf930bfdab6ab852'
subprocess.run(['python3',str(root/'scripts/native-peer.py')],check=True)
out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs','gz-math'],text=True))
libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(root/'tests/enum_reference.cc'),*flags,'-Wl,-rpath,'+libdir,'-o',str(out/'enum-reference')],check=True)
files=descriptor_pb2.FileDescriptorSet.FromString((root/'target/native/messages.pb').read_bytes())
pool=descriptor_pool.DescriptorPool()
for file in files.file:pool.AddSerializedFile(file.SerializeToString())
rows=[]
for family,name in [('Joint','gz.msgs.Joint.Type'),('Geometry','gz.msgs.Geometry.Type'),('Shader','gz.msgs.Material.ShaderType'),('PixelFormat','gz.msgs.PixelFormatType'),('Discovery','gz.msgs.Discovery.Type')]:
    desc=pool.FindEnumTypeByName(name);names=set()
    for number in [v.number for v in desc.values]+[-1,100]:
        result=subprocess.run([str(out/'enum-reference'),family,'format',str(number)],check=True,capture_output=True,text=True)
        rows.append({'family':family,'direction':'format','number':number,'name':result.stdout,'stderr':result.stderr});names.add(result.stdout)
    names|={v.name for v in desc.values}|{v.name.lower() for v in desc.values}|{'bad type',''}
    for name in sorted(names):
        result=subprocess.run([str(out/'enum-reference'),family,'parse',name],check=True,capture_output=True,text=True)
        rows.append({'family':family,'direction':'parse','name':name,'number':int(result.stdout),'stderr':result.stderr})
headers={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (reference/'core/include/gz/msgs/convert').glob('*Type.hh')}
fixture={'upstream_commit':'89c0c3d1474f545cb889335daf930bfdab6ab852','header_sha256':headers,'native_binary_sha256':hashlib.sha256((out/'enum-reference').read_bytes()).hexdigest(),'cases':rows}
(root/'tests/fixtures/enum-reference.json').write_text(json.dumps(fixture,indent=2)+'\n')
print(len(rows),'native enum conversion cases captured')
