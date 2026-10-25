// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/time.pb.h>
#include <gz/msgs/pose.pb.h>
#include <gz/msgs/parameter.pb.h>
#include <gz/msgs/param.pb.h>
#include <gz/msgs/any.pb.h>
#include <gz/msgs/double_v.pb.h>
#include <google/protobuf/message.h>
#include <iostream>
#include <iterator>
#include <memory>
int main(int argc,char **argv){
 if(argc<2)return 2;
 // Force these generated descriptor registrations even with --as-needed linking.
 const google::protobuf::Descriptor* used[]={gz::msgs::Time::descriptor(),gz::msgs::Pose::descriptor(),gz::msgs::Parameter::descriptor(),gz::msgs::Param::descriptor(),gz::msgs::Any::descriptor(),gz::msgs::Double_V::descriptor()};
 (void)used;
 auto d=google::protobuf::DescriptorPool::generated_pool()->FindMessageTypeByName(argv[1]);if(!d)return 3;
 std::unique_ptr<google::protobuf::Message> m(google::protobuf::MessageFactory::generated_factory()->GetPrototype(d)->New());
 std::string input(std::istreambuf_iterator<char>(std::cin),{});
 if(!m->ParseFromString(input))return 6;
 if(argc==3)m->DiscardUnknownFields();
 if(!m->SerializeToOstream(&std::cout))return 7;
}
