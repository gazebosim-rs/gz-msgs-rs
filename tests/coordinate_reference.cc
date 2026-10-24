// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/Utility.hh>
#include <cstdlib>
#include <iostream>
int main(int argc, char **argv)
{
  if (argc != 3) return 1;
  const int value=std::stoi(argv[2]);
  if (std::string(argv[1])=="from_math")
    std::cout << static_cast<int>(gz::msgs::ConvertCoord(static_cast<gz::math::SphericalCoordinates::CoordinateType>(value))) << '\n';
  else if (std::string(argv[1])=="to_math")
    std::cout << static_cast<int>(gz::msgs::Convert(static_cast<gz::msgs::SphericalCoordinatesType>(value))) << '\n';
  else return 2;
}
