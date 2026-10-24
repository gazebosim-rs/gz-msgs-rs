// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Original retry assertions: factory_descriptor_retry.cc, OSRF 2026.
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <google/protobuf/descriptor.pb.h>
#include <gz/msgs/MessageFactory.hh>
void hex(const std::string &bytes) {
  for (unsigned char byte:bytes) std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<static_cast<int>(byte);
}
int main(int argc,char **argv) {
  if (argc!=2) return 2;
  namespace pb=google::protobuf;
  pb::FileDescriptorSet a,b;
  auto bf=b.add_file();bf->set_name("b.proto");bf->set_package("b.msgs");bf->set_syntax("proto3");
  auto bar=bf->add_message_type();bar->set_name("Bar");auto x=bar->add_field();x->set_name("x");x->set_number(1);x->set_type(pb::FieldDescriptorProto::TYPE_INT32);x->set_label(pb::FieldDescriptorProto::LABEL_OPTIONAL);
  auto af=a.add_file();af->set_name("a.proto");af->set_package("a.msgs");af->set_syntax("proto3");af->add_dependency("b.proto");
  auto foo=af->add_message_type();foo->set_name("Foo");auto field=foo->add_field();field->set_name("bar");field->set_number(1);field->set_type(pb::FieldDescriptorProto::TYPE_MESSAGE);field->set_type_name(".b.msgs.Bar");field->set_label(pb::FieldDescriptorProto::LABEL_OPTIONAL);
  const auto dir=std::filesystem::path(argv[1]);std::filesystem::create_directories(dir);
  {std::ofstream out(dir/"a.desc",std::ios::binary);if (!a.SerializeToOstream(&out)) return 3;}
  {std::ofstream out(dir/"b.desc",std::ios::binary);if (!b.SerializeToOstream(&out)) return 3;}
  gz::msgs::MessageFactory factory;
  factory.LoadDescriptors((dir/"a.desc").string());bool missing=!factory.New("a.msgs.Foo");
  factory.LoadDescriptors((dir/"b.desc").string());bool dependency=static_cast<bool>(factory.New("b.msgs.Bar"));
  factory.LoadDescriptors((dir/"a.desc").string());bool retry=static_cast<bool>(factory.New("a.msgs.Foo"));
  if (!missing || !dependency || !retry) return 4;
  std::cout<<"{\"a_hex\":\"";hex(a.SerializeAsString());std::cout<<"\",\"b_hex\":\"";hex(b.SerializeAsString());
  std::cout<<"\",\"initial_missing\":true,\"dependency_loaded\":true,\"retry_succeeded\":true}";
}
