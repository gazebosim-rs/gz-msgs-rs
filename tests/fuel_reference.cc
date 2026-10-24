// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/convert/FuelMetadata.hh>
#include <iostream>
#include <iterator>
#include <iomanip>
void hex(const std::string &value) {
  for (unsigned char c : value) std::cout << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(c);
}
int main() {
  std::string input{std::istreambuf_iterator<char>(std::cin), {}};
  gz::msgs::FuelMetadata meta; meta.set_name("retained");
  try {
    bool parsed = gz::msgs::ConvertFuelMetadata(input, meta);
    std::string output = "retained";
    bool written = parsed && gz::msgs::ConvertFuelMetadata(meta, output);
    std::cout << parsed << " " << written << " "; hex(meta.SerializeAsString()); std::cout << " "; hex(output);
  } catch (const std::exception &e) { std::cout << "exception " << e.what(); }
}
