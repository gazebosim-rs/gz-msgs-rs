// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <google/protobuf/descriptor.pb.h>
#include <google/protobuf/descriptor.h>
#include <fstream>
#include <iostream>
int main(int argc,char**argv) {
  if(argc!=2)return 2;
  google::protobuf::FileDescriptorSet set;
  std::ifstream input(argv[1],std::ios::binary);
  if(!set.ParseFromIstream(&input))return 3;
  google::protobuf::DescriptorPool pool;
  for(const auto &file:set.file())if(!pool.BuildFile(file))return 4;
  auto message=pool.FindMessageTypeByName("Value");if(!message)return 5;
  auto field=message->FindFieldByName("value");if(!field)return 6;
  std::cout<<field->default_value_string();
}
