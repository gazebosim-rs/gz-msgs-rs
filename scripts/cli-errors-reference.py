#!/usr/bin/env python3
# Copyright (C) 2026 Open Source Robotics Foundation
# SPDX-License-Identifier: Apache-2.0
"""Capture native info failure/continuation/alias behavior without external schemas."""
import hashlib,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parents[1];reference=root.parent/'gz-msgs'
revision='89c0c3d1474f545cb889335daf930bfdab6ab852'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=reference,text=True).strip()==revision
sources=['core/cmd/msgs_main.cc','core/src/MessageFactory.cc']
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
for path in sources:
 assert hashlib.sha256(subprocess.check_output(['git','show',revision+':'+path],cwd=reference)).hexdigest()==sha(reference/path)
binary=reference/'build/bin/gz-msgs';env=dict(os.environ);env.pop('GZ_DESCRIPTOR_PATH',None)
rows=[]
for args in [['--info','missing'],['--info','gz.msgs.Time','missing','gz.msgs.StringMsg'],['--info','gz.msgs.PixelFormatType'],['--info',''],['--info','missing','missing'],['--info','..gz.msgs.Time','.google.protobuf.Any','google.protobuf.Any'],['--info','.gz.msgs.Time','.gz_msgs.Time'],['-i','missing'],['--info','snow☃\nname']]:
 run=subprocess.run([str(binary),*args],env=env,capture_output=True,text=True,check=True)
 rows.append(dict(args=args,exit_code=run.returncode,stdout=run.stdout,stderr=run.stderr))
proof=dict(upstream_commit=revision,source_sha256={p:sha(reference/p) for p in sources},native_binary_sha256=sha(binary),rows=rows)
(root/'tests/fixtures/cli-errors-reference.json').write_text(json.dumps(proof,indent=2)+'\n')
print(len(rows),'native info command observations captured')
