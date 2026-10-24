#!/usr/bin/env python3
"""Run the unchanged upstream C++ CLI assertions against the Rust executable."""
import pathlib, subprocess, sys, tomllib
root=pathlib.Path(__file__).resolve().parents[1]
out=root/'target/rehost-cli'
config=out/'include/gz/msgs'
config.mkdir(parents=True,exist_ok=True)
version=tomllib.loads((root/'Cargo.toml').read_text())['package']['version']
# Bind this package's generated version exactly as upstream config.hh does.
(config/'config.hh').write_text('#define GZ_MSGS_VERSION_FULL "'+version+'"\n')
subprocess.run(['c++','-std=c++17','-I'+str(out/'include'),'-I'+str(root/'tests/reference/include'),str(root/'tests/reference/gz_TEST.cc'),'-DGZ_MSGS_EXECUTABLE_PATH="'+str(pathlib.Path(sys.argv[1]).resolve())+'"','-DGZ_MSGS_COMPLETION_SCRIPT_PATH="'+str(root/'scripts/msgs.bash_completion.sh')+'"','-lgtest_main','-lgtest','-pthread','-o',str(out/'cli-reference')],check=True)
subprocess.run([str(out/'cli-reference'),'--gtest_output=xml:'+str(out/'results.xml')],check=True)
