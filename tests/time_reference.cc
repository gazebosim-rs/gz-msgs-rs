// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Native reference only: compile against the pinned Gazebo conversion header.
#include <gz/msgs/convert/StdTypes.hh>
#include <chrono>
#include <iomanip>
#include <iostream>
int main(int argc, char **argv) {
  if (argc == 3 && std::string(argv[1]) == "encode") {
    auto nanos = std::chrono::nanoseconds(std::stoll(argv[2]));
    auto message = gz::msgs::Convert(std::chrono::steady_clock::duration(nanos));
    std::cout << message.sec() << " " << message.nsec() << " ";
    for (unsigned char byte : message.SerializeAsString())
      std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(byte);
    std::cout << "\n";
  } else if (argc == 4 && std::string(argv[1]) == "decode") {
    gz::msgs::Time message;
    message.set_sec(std::stoll(argv[2]));
    message.set_nsec(std::stoi(argv[3]));
    std::cout << std::chrono::duration_cast<std::chrono::nanoseconds>(gz::msgs::Convert(message)).count() << "\n";
  } else return 2;
}
