// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Original assertions: Copyright (C) 2024 Open Source Robotics Foundation.
#include <filesystem>
#include <iostream>
#include <gz/msgs/MessageFactory.hh>
using Names=std::vector<std::string>;
void state(gz::msgs::MessageFactory &factory,const std::string &label,const Names &names,bool comma) {
  if(comma) std::cout<<",";
  std::cout<<"{\"stage\":\""<<label<<"\",\"types\":[";
  for(size_t i=0;i<names.size();++i){if(i)std::cout<<",";std::cout<<"\""<<names[i]<<"\"";}
  std::cout<<"],\"present\":[";
  for(size_t i=0;i<names.size();++i){if(i)std::cout<<",";std::cout<<(factory.New(names[i])?"true":"false");}
  std::cout<<"]}";
}
int main(int argc,char **argv) {
  if(argc!=3)return 2;
  const auto dir=std::filesystem::path(argv[1]);gz::msgs::MessageFactory factory;
  Names common={"example.msgs.StringMsg","testing.Bytes","testing.FooMessage","testing.BarMessage","testing.BazMessage"};
  std::cout<<"{\"stages\":[";state(factory,"initial",common,false);
  factory.LoadDescriptors((dir/"does_not_exist.desc").string());state(factory,"missing",common,true);
  factory.LoadDescriptors((dir/"testing.invalid").string());state(factory,"extension",common,true);
  factory.LoadDescriptors((dir/"testing.proto").string());
  state(factory,"content",{"example.msgs.StringMsg","testing.BarBytes","testing.FooMessage","testing.BarMessage","testing.BazMessage"},true);
  factory.LoadDescriptors((dir/"stringmsg.desc").string());state(factory,"file",common,true);
  factory.LoadDescriptors(dir.string());
  state(factory,"directory",{"example.msgs.StringMsg","testing.FooBytes","testing.FooMessage","testing.BarMessage","testing.BazMessage"},true);
  const auto path=(dir/"stringmsg.desc").string();factory.LoadDescriptors(path+":"+path);
  const auto protectedDir=std::filesystem::path(argv[2]);
  std::filesystem::create_directories(protectedDir);
  std::filesystem::copy_file(dir/"stringmsg.desc",protectedDir/"gz-msgs.gz_desc",std::filesystem::copy_options::overwrite_existing);
  std::filesystem::permissions(protectedDir,std::filesystem::perms::owner_exec|std::filesystem::perms::group_exec|std::filesystem::perms::others_exec);
  gz::msgs::MessageFactory protectedFactory;bool listingFailed=false;
  try {protectedFactory.LoadDescriptors(protectedDir.string());} catch(const std::filesystem::filesystem_error &) {listingFailed=true;}
  const bool ownLoaded=static_cast<bool>(protectedFactory.New("example.msgs.StringMsg"));
  std::filesystem::permissions(protectedDir,std::filesystem::perms::owner_all);
  std::cout<<"],\"duplicate_present\":"<<(factory.New("example.msgs.StringMsg")?"true":"false")<<",\"execute_only_directory\":{\"listing_failed\":"<<(listingFailed?"true":"false")<<",\"own_loaded\":"<<(ownLoaded?"true":"false")<<"}}";
}
