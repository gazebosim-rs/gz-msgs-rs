// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <google/protobuf/descriptor.h>
#include <google/protobuf/descriptor.pb.h>
#include <google/protobuf/descriptor_database.h>
#include <google/protobuf/unknown_field_set.h>
#include <fstream>
#include <iostream>
void print(const google::protobuf::Descriptor *d){
 std::cout<<d->full_name()<<' ';
 for(unsigned char c:d->DebugString()){const char *h="0123456789abcdef";std::cout<<h[c/16]<<h[c%16];}
 std::cout<<'\n';for(int i=0;i<d->nested_type_count();++i)print(d->nested_type(i));
}
int main(int argc,char**argv){if(argc!=2 && argc!=3)return 2;
 std::ifstream input(argv[1],std::ios::binary);google::protobuf::FileDescriptorSet set;if(!set.ParseFromIstream(&input))return 3;
 if(argc==3){
  if(!set.file_size() || !set.file(set.file_size()-1).message_type_size())return 6;
  auto options=set.mutable_file(set.file_size()-1)->mutable_message_type(0)->mutable_options();
  options->GetReflection()->MutableUnknownFields(options)->AddVarint(900001,1);
  std::ofstream output(argv[2],std::ios::binary);return set.SerializeToOstream(&output)?0:7;
 }
 google::protobuf::SimpleDescriptorDatabase database;for(const auto&f:set.file())if(!database.Add(f))return 4;
 google::protobuf::DescriptorPool pool(&database);
 for(const auto&f:set.file()){auto file=pool.FindFileByName(f.name());if(!file)return 5;for(int i=0;i<file->message_type_count();++i)print(file->message_type(i));}
}
