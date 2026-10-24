// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Native reference only: metadata initialization, not unsafe iterator dereferences.
#include <iostream>
#include <iomanip>
#include <gz/msgs/PointCloudPackedUtils.hh>
int main(int argc, char **argv) {
  using Cloud=gz::msgs::PointCloudPacked;
  if (argc==2 && std::string(argv[1])=="word") { std::cout << sizeof(size_t); return 0; }
  if (argc==3 && std::string(argv[1])=="size") {
    std::cout << gz::msgs::sizeOfPointField(static_cast<Cloud::Field::DataType>(std::stoi(argv[2])));
    return 0;
  }
  if (argc<4 || (argc-4)%2) return 2;
  std::vector<std::pair<std::string,Cloud::Field::DataType>> fields;
  for (int index=4;index<argc;index+=2) fields.emplace_back(argv[index],static_cast<Cloud::Field::DataType>(std::stoi(argv[index+1])));
  Cloud message;
  message.set_width(3);message.set_height(2);message.set_row_step(333);
  message.set_is_bigendian(true);message.set_is_dense(true);message.set_data("kept");
  message.mutable_header()->mutable_stamp()->set_sec(42);
  message.mutable_header()->mutable_stamp()->set_nsec(17);
  auto old=message.mutable_header()->add_data();old->set_key("old");old->add_value("oldValue");
  message.add_field()->set_name("oldField");
  gz::msgs::InitPointCloudPacked(message,argv[3],std::string(argv[2])=="1",fields);
  for (unsigned char byte:message.SerializeAsString()) std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(byte);
}
