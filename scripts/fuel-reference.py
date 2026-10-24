#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture native Fuel XML behavior and exact serialized metadata/output bytes."""
import hashlib,json,pathlib,re,shlex,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
reference=root.parent/'gz-msgs'; math=root.parent/'gz-math'
revision='89c0c3d1474f545cb889335daf930bfdab6ab852'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()==revision
out=root/'target/reference';out.mkdir(parents=True,exist_ok=True)
flags=shlex.split(subprocess.check_output(['pkg-config','--cflags','--libs','gz-msgs','gz-math','tinyxml2'],text=True))
libdir=subprocess.check_output(['pkg-config','--variable=libdir','gz-msgs'],text=True).strip()
binary=out/'fuel-reference';source=root/'tests/fuel_reference.cc'
subprocess.run(['c++','-std=c++17','-O2','-I'+str(reference/'core/include'),str(source),*flags,'-Wl,-rpath,'+libdir,'-o',str(binary)],check=True)
original=(reference/'test/integration/Utility_TEST.cc').read_text()
model=re.search(r'modelConfigInput = R"\((<\?xml.*?)\)";',original,re.S)[1]
world=re.search(r'worldConfigInput = R"\((<\?xml.*?)\)";',original,re.S)[1]
cases=[('empty',''),('wrong_root','<test/>'),('no_name','<model>test</model>'),('no_sdf','<model><name>test_model</name></model>'),('original_model',model),('original_world',world)]
def wrap(name='name',version='1.7',file='model.sdf',extra=''):
 return f'<model><name>{name}</name><sdf version="{version}">{file}</sdf>{extra}</model>'
for i,name in enumerate(['',' ','<![CDATA[]]>','<![CDATA[ ]]>','<!-- comment -->','x<child/>after','<child/>after','x<!-- c -->later','  x  ','&#32;','&#32;x  ','\u00a0','x&unknown;y','&#0;x','x\r\ny','x\n\ry','<![CDATA[x\n\ry]]>','<![CDATA[x\r\ny]]>','x<![CDATA[y]]>z','<!-- c -->x','<?pi x?>y','é名','x&dangling','<!DOCTYPE x>after','&#0000;tail','&#x00;tail','\u000b','\u000c','&#11;','&#12;']):
 cases.append(('text_'+str(i),wrap(name)))
for i,version in enumerate(['','0','0.0','0.0.0','0.0.0-alpha','0.0.1','1','1.7','1.7.3-alpha+build','01.02.03','-1.2','1.2.3.4','1x.7tail','1.2.','1..2','word','2147483648','1+build-with-hyphen','1.2.3-','1.2.3+','1.2..','1.2.3.4.5','+build','1.2.3-a.b+c.d','1.7&unknown;','1.7&#13;','1.7\r\n','1.7&#0;.9','1.7&#0000;.9','1.7&#x00;.9','1.7&dangling','1.7&#bad;']):
 cases.append(('version_'+str(i),wrap(version=version)))
for i,value in enumerate(['-1','+3suffix','2147483647','2147483648','','  ','word']):
 cases.append(('resource_version_'+str(i),wrap(extra=f'<version>{value}</version>')))
for i,extra in enumerate(['<author/>','<author><name> A </name><email> B </email></author><author/>','<depend/>','<depend><world><uri>x</uri></world></depend>','<depend><model><uri> model://raw </uri></model></depend>','<sdf version="1.6">last.sdf</sdf>','<sdf version="0">ignored.sdf</sdf>','<sdf version="1.6"> </sdf>','<description> x &amp; y </description>','<name>second</name>']):
 cases.append(('extra_'+str(i),wrap(extra=extra)))
cases.extend([('multiple_roots',world+model),('prefixed','<p:model><name>x</name><sdf version="1.7">m</sdf></p:model>'),('namespace',wrap().replace('<model>','<model xmlns="ignored">')),('nul_tail',wrap()+'\0<bad'),('bad_xml',wrap()[:-1]),('empty_file',wrap(file='<![CDATA[]]>')),('true_multiple_roots',world+model.replace("<?xml version='1.0'?>","")),('decl_after_comment','<!-- c -->'+model),('uri_whitespace',wrap(extra='<depend><model><uri>  &amp;x </uri></model></depend>'))])
for depth in [496,497,498,499]:
 cases.append(("depth_"+str(depth),"<model><name>n</name><sdf version=\"1.7\">m</sdf>"+"<x>"*depth+"</x>"*depth+"</model>"))
for name,xml in [
 ('duplicate_attr',wrap().replace('version="1.7"','version="1.7" version="1.6"')),
 ('unquoted_attr',wrap().replace('version="1.7"','version=1.7')),
 ('bad_end',wrap().replace('</name>','</wrong>')),
 ('missing_close','<model>'),
 ('attr_escapes',wrap().replace('version="1.7"','version="1.7" ignored="&amp;&quot;&unknown;"')),
 ('numeric_ref_empty',wrap('&#x;')),
 ('numeric_ref_bad',wrap('&#bad;')),
 ('invalid_char','<model><name>n</name><sdf version="1.7">m</sdf><')]:
 cases.append((name,xml))
import itertools
for size in range(1,5):
 for i,seq in enumerate(itertools.product('\r\n',repeat=size)):
  cases.append(('newlines_'+str(size)+'_'+str(i),wrap('left'+''.join(seq)+'right')))
cases.extend([('review_leading_text', "prefix<model><name>n</name><sdf version='1.7'>m</sdf></model>"),
 ('review_trailing_text', "<model><name>n</name><sdf version='1.7'>m</sdf></model>suffix"),
 ('review_bad_child_start', "<model><name>n</name><sdf version='1.7'>m</sdf><1bad/></model>"),
 ('review_bad_child_name', "<model><name>n</name><sdf version='1.7'>m</sdf><bad!/></model>"),
 ('review_bad_attr_name', "<model><name>n</name><sdf version='1.7'>m</sdf><unused 1bad='x'/></model>"),
 ('review_empty_prefix', "<model><name>n</name><sdf version='1.7'>m</sdf><:/></model>"),
 ('review_prolog_bad_pi', "<?1bad?><model><name>n</name><sdf version='1.7'>m</sdf></model>"),
 ('review_prolog_pi', "<?pi x?><model><name>n</name><sdf version='1.7'>m</sdf></model>"),
 ('review_name_ref_splits', "<model><name> &amp;  </name><sdf version='1.7'>m</sdf></model>"),
 ('review_comment_weird', "<model><name>n</name><sdf version='1.7'>m</sdf><!-- a -- b --></model>"),
 ('review_literal_controls', "<model><name>\x01n</name><sdf version='1.7'>m</sdf></model>")])
for i,suffix in enumerate([' &amp;','&#32;','suffix<!--end-->','suffix<extra/>','<!--end-->','<![CDATA[tail]]>','\t\r\n','suffix<![CDATA[tail]]>','suffix&#32;',' &dangling']):
 cases.append(('terminal_'+str(i),wrap()+suffix))
for i,n in enumerate([':','_','é','a:b','a.b-c0','.bad','-bad','bad!']):
 cases.append(('element_name_'+str(i),wrap(extra='<'+n+'/>')))
 cases.append(('attribute_name_'+str(i),wrap(extra='<unused '+n+'="x"/>')))
rows=[]
for name,xml in cases:
 run=subprocess.run([str(binary)],input=xml,text=True,capture_output=True,check=True)
 parts=run.stdout.split(' ')
 row=dict(id=name,xml=xml,stderr=run.stderr)
 if parts[0]=='exception':row['exception']=' '.join(parts[1:])
 else:row.update(parsed=parts[0]=='1',written=parts[1]=='1',wire_hex=parts[2],xml_hex=parts[3])
 rows.append(row)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
paths=['core/include/gz/msgs/convert/FuelMetadata.hh','test/integration/Utility_TEST.cc']
for path in paths:
 assert hashlib.sha256(subprocess.check_output(['git','show',revision+':'+path],cwd=reference)).hexdigest()==sha(reference/path)
math_revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=math,text=True).strip()
assert hashlib.sha256(subprocess.check_output(['git','show',math_revision+':src/SemanticVersion.cc'],cwd=math)).hexdigest()==sha(math/'src/SemanticVersion.cc')
linked=subprocess.check_output(['ldd',str(binary)],text=True)
libraries={line.strip().split()[0]:pathlib.Path(line.split('=>',1)[1].split()[0]) for line in linked.splitlines() if '=>' in line and line.split('=>',1)[1].strip().startswith('/')}
math_library=next(p for n,p in libraries.items() if n.startswith('libgz-math.so.'))
assert sha(math_library)==sha(math/'build/lib'/math_library.name)
original_binary=reference/'build/bin/INTEGRATION_Utility_TEST';report=out/'fuel-original.xml'
run=subprocess.run([str(original_binary),'--gtest_filter=UtilityTest.ConvertFuelMetadata','--gtest_output=xml:'+str(report)],check=True,capture_output=True,text=True)
import xml.etree.ElementTree as ET
cases_xml=[test for suite in ET.parse(report).getroot() for test in suite.findall('testcase')]
assert len(cases_xml)==1 and cases_xml[0].attrib['name']=='ConvertFuelMetadata' and cases_xml[0].attrib.get('status')=='run' and not list(cases_xml[0])
(out/'fuel-original.log').write_text(run.stdout+run.stderr)
fixture=dict(native_libraries_sha256={n:sha(p) for n,p in libraries.items() if n.startswith(('libgz-math.so.','libtinyxml2.so.'))},tinyxml_header_sha256=sha(pathlib.Path('/usr/include/tinyxml2.h')),original_binary_sha256=sha(original_binary),upstream_commit=revision,source_sha256={p:sha(reference/p) for p in paths},probe_source_sha256=sha(source),probe_binary_sha256=sha(binary),math_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=math,text=True).strip(),math_source_sha256=sha(math/'src/SemanticVersion.cc'),rows=rows)
(root/'tests/fixtures/fuel-reference.json').write_text(json.dumps(fixture,indent=2)+'\n')
print(len(rows),'native Fuel XML observations captured; C++ exceptions recorded separately')
