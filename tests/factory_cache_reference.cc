// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/MessageFactory.hh>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <string>
namespace fs=std::filesystem;
std::string path(const std::string &kind) {
 if(kind=="relative")return "a.desc";
 if(kind=="dot")return "./a.desc";
 if(kind=="parent")return "sub/../a.desc";
 if(kind=="absolute")return fs::absolute("a.desc").string();
 if(kind=="absolute-dot")return fs::absolute("./a.desc").string();
 if(kind=="absolute-parent")return fs::absolute("sub/../a.desc").string();
 if(kind=="raw-other")return "raw-\xfe.desc";
 if(kind=="replacement")return "raw-\xef\xbf\xbd.desc";
 if(kind=="raw")return "raw-\xff.desc";
 if(kind=="raw-absolute")return fs::absolute("raw-\xff.desc").string();
 if(kind=="nested")return "sub/a.desc";
 if(kind=="nested-dot")return "./sub/a.desc";
 if(kind=="nested-absolute")return fs::absolute("sub/a.desc").string();
 if(kind=="nested-absolute-dot")return fs::absolute("./sub/a.desc").string();
 if(kind=="nested-absolute-parent")return fs::absolute("gone/../sub/a.desc").string();
 if(kind=="directory-alias")return "dir-alias/a.desc";
 if(kind=="directory-alias-absolute")return fs::absolute("dir-alias/a.desc").string();
 if(kind=="directory-alias-parent")return "deep-alias/../a.desc";
 if(kind=="trailing")return "a.desc/";
 if(kind=="absolute-trailing")return fs::absolute("a.desc/").string();
 if(kind=="trailing-dot")return "a.desc/.";
 if(kind=="notdir")return "a.desc/../a.desc";
 if(kind=="missing-parent")return "gone/../a.desc";
 if(kind=="absolute-missing-parent")return fs::absolute("gone/../a.desc").string();
 if(kind=="alias")return "alias.desc";
 if(kind=="absolute-alias")return fs::absolute("alias.desc").string();
 throw std::runtime_error("unknown path kind");
}
void state(gz::msgs::MessageFactory &factory,int step) {
 std::cout<<"STATE "<<step;
 for(auto name:{"audit.A","audit.B","audit.C","audit.Dep"})std::cout<<' '<<bool(factory.New(name));
 std::cout<<'\n';
}
int main(int argc,char **argv) {
 if(argc!=5)return 2;
 fs::create_directory("sub");fs::create_symlink("a.desc","alias.desc");
 const fs::path inputs=argv[4];const std::string action=argv[1];
 auto copy=[&](const std::string &from,const std::string &to){fs::copy_file(inputs/from,to,fs::copy_options::overwrite_existing);};
 if(action=="malformed") {std::ofstream out("a.desc",std::ios::binary);out.put('\xff');}
 else if(action=="partial")copy("partial.desc","a.desc");
 else if(action=="empty")copy("empty.desc","a.desc");
 else if(action!="missing" && action!="dangling")copy("A.desc","a.desc");
 if(action=="raw-replace"||action=="raw-delete"||action=="raw-distinct")copy("A.desc",path("raw"));
 if(action=="delete-parent"||action=="permissions")copy("A.desc","sub/a.desc");
 if(action=="dangling-directory") {fs::create_symlink("sub","dir-alias");copy("A.desc","sub/a.desc");}
 if(action=="symlink-parent") {fs::create_directory("sub/deep");fs::create_symlink("sub/deep","deep-alias");copy("A.desc","sub/a.desc");}
 gz::msgs::MessageFactory factory;
 auto load=[&](const std::string &name,int step){auto p=path(name);std::error_code err;auto key=fs::weakly_canonical(p,err);std::cout<<"KEY "<<step<<' '<<std::quoted(p)<<' '<<std::quoted(err?p:key.string())<<'\n';std::cout.flush();factory.LoadDescriptors(p);state(factory,step);};
 load(argv[2],1);
 if(action=="delete")fs::remove("a.desc");
 else if(action=="raw-distinct")copy("B.desc",path(argv[3]));
 else if(action=="raw-replace")copy("B.desc",path("raw"));
 else if(action=="raw-delete")fs::remove(path("raw"));
 else if(action=="delete-parent"||action=="dangling-directory")fs::remove_all("sub");
 else if(action=="permissions")fs::permissions("sub",fs::perms::none);
 else if(action=="symlink-parent")fs::remove("sub/a.desc");
 else if(action=="cached-loop") {fs::remove("a.desc");fs::create_symlink("a.desc","a.desc");}
 else if(action=="loop") {fs::remove("alias.desc");fs::create_symlink("loop.desc","alias.desc");fs::create_symlink("alias.desc","loop.desc");}
 else if(action=="retarget"){copy("B.desc","b.desc");fs::remove("alias.desc");fs::create_symlink("b.desc","alias.desc");}
 else if(action=="partial"){copy("Dep.desc","dep.desc");factory.LoadDescriptors("dep.desc");state(factory,2);}
 else if(action=="missing"||action=="dangling"||action=="malformed")copy("A.desc","a.desc");
 else copy("B.desc","a.desc");
 load(argv[3],3);
 if(action=="permissions")fs::permissions("sub",fs::perms::owner_all);
}
