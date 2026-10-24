// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <algorithm>
#include <iomanip>
#include <iostream>
#include <gz/msgs/Factory.hh>
#include <gz/msgs/vector3d.pb.h>
int main() {
  std::vector<std::string> types;gz::msgs::Factory::Types(types);std::sort(types.begin(),types.end());
  std::cout<<"{\"types\":[";
  bool first=true;
  for (const auto &type:types) {
    if (!gz::msgs::Factory::New(type)) return 2;
    if (!first) std::cout<<",";first=false;std::cout<<"\""<<type<<"\"";
  }
  auto valid=gz::msgs::Factory::New<gz::msgs::Vector3d>("gz.msgs.Vector3d","x: 1.0, y: 2.0, z: 3.0");
  if (!valid) return 3;
  std::cout<<"],\"valid_wire_hex\":\"";
  for (unsigned char byte:valid->SerializeAsString()) std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<static_cast<int>(byte);
  std::cout<<"\",\"malformed_is_null\":"<<(gz::msgs::Factory::New<gz::msgs::Vector3d>("gz.msgs.Vector3d","x: 1.0, y: asdf, z: 3.0")?"false":"true")<<"}";
}
