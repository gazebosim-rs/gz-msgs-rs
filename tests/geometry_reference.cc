// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/Utility.hh>
#include <array>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>

std::string Hex(const google::protobuf::Message &_message)
{
  std::string wire;
  if (!_message.SerializeToString(&wire)) std::abort();
  std::ostringstream out;
  for (unsigned char byte : wire)
    out << std::hex << std::setw(2) << std::setfill('0') << unsigned(byte);
  return out.str();
}
int main()
{
  const std::array<std::array<double, 6>, 4> planes{{
    {0, 0, 1, 123, 456, 1}, {-2, 3, 4, -5, 6, -7},
    {-0., 0, 1, -0., 2, -0.}, {0, 0, 0, 0, 0, 0}}};
  const std::array<std::array<double, 6>, 4> boxes{{
    {-1, -2, -3, 12, 15, 20}, {-23.4, 56.2, -123.5, 34.1, 92.1, -723.2},
    {0, 0, 0, 0, 0, 0}, {-0., 2, 3, 4, -0., 6}}};
  std::cout << "[";
  bool first = true;
  for (int kind = 0; kind < 2; ++kind)
    for (int index = 0; index < 4; ++index)
      for (int seeded = 0; seeded < 2; ++seeded)
      {
        std::string wire;
        if (kind == 0)
        {
          gz::msgs::PlaneGeom message;
          if (seeded)
          {
            message.mutable_header()->mutable_stamp()->set_sec(11);
            message.mutable_normal()->mutable_header()->mutable_stamp()->set_sec(22);
            message.mutable_size()->mutable_header()->mutable_stamp()->set_sec(33);
          }
          const auto &p = planes[index];
          gz::msgs::Set(&message, gz::math::Planed({p[0], p[1], p[2]}, {p[3], p[4]}, p[5]));
          const auto value = gz::msgs::Convert(message);
          if (value.Normal() != gz::math::Vector3d(p[0], p[1], p[2]) ||
              value.Size() != gz::math::Vector2d(p[3], p[4]) || value.Offset() != p[5]) return 1;
          wire = Hex(message);
        }
        else
        {
          gz::msgs::AxisAlignedBox message;
          if (seeded)
          {
            message.mutable_header()->mutable_stamp()->set_sec(11);
            message.mutable_min_corner()->mutable_header()->mutable_stamp()->set_sec(22);
            message.mutable_max_corner()->mutable_header()->mutable_stamp()->set_sec(33);
          }
          const auto &b = boxes[index];
          // Use mutable corners, as the original test does: construction sorts them.
          gz::math::AxisAlignedBox value;
          value.Min() = {b[0], b[1], b[2]};
          value.Max() = {b[3], b[4], b[5]};
          gz::msgs::Set(&message, value);
          const auto copy = gz::msgs::Convert(message);
          if (copy.Min() != value.Min() || copy.Max() != value.Max()) return 1;
          wire = Hex(message);
        }
        if (!first) std::cout << ",";
        first = false;
        std::cout << "{\"kind\":" << kind << ",\"index\":" << index
                  << ",\"seeded\":" << seeded << ",\"wire_hex\":\"" << wire << "\"}";
      }
  const auto plane = gz::msgs::Convert(gz::msgs::PlaneGeom());
  const auto box = gz::msgs::Convert(gz::msgs::AxisAlignedBox());
  if (plane.Normal() != gz::math::Vector3d::Zero || plane.Size() != gz::math::Vector2d::Zero ||
      plane.Offset() != 0 || box.Min() != gz::math::Vector3d::Zero || box.Max() != gz::math::Vector3d::Zero) return 1;
  std::cout << "]\n";
}
