// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/Utility.hh>
#include <array>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>
std::string Hex(const google::protobuf::Message &_message)
{
  std::string bytes;
  if (!_message.SerializeToString(&bytes)) std::abort();
  std::ostringstream out;
  for (unsigned char byte : bytes) out << std::hex << std::setw(2) << std::setfill('0') << unsigned(byte);
  return out.str();
}
std::uint64_t Bits(double _value)
{
  std::uint64_t result;
  std::memcpy(&result, &_value, sizeof(result));
  return result;
}
int main()
{
  const std::array<gz::math::Quaterniond, 3> rotations{{
    {GZ_PI * 0.25, GZ_PI * 0.5, GZ_PI}, {4, 1, 2, 3}, {0, 0, 0, 0}}};
  bool first = true;
  std::cout << "{\"orientations\":[";
  for (const auto &q : rotations)
  {
    if (!first) std::cout << ",";
    first = false;
    std::cout << "[" << Bits(q.X()) << "," << Bits(q.Y()) << "," << Bits(q.Z()) << "," << Bits(q.W()) << "]";
  }
  std::cout << "],\"rows\":[";
  first = true;
  auto emit = [&](const std::string &kind, int index, int seeded, const auto &message)
  {
    if (!first) std::cout << ",";
    first = false;
    std::cout << "{\"kind\":\"" << kind << "\",\"index\":" << index
              << ",\"seeded\":" << seeded << ",\"wire_hex\":\"" << Hex(message) << "\"}";
  };
  for (int seeded = 0; seeded < 2; ++seeded)
  {
    gz::msgs::Vector2d v2;
    gz::msgs::Vector3d v3;
    gz::msgs::Color color;
    if (seeded)
    {
      v2.mutable_header()->mutable_stamp()->set_sec(11);
      v3.mutable_header()->mutable_stamp()->set_sec(11);
      color.mutable_header()->mutable_stamp()->set_sec(11);
    }
    gz::msgs::Set(&v2, gz::math::Vector2d(1, 2));
    gz::msgs::Set(&v3, gz::math::Vector3d(1, 2, 3));
    gz::msgs::Set(&color, gz::math::Color(.1f, .2f, .3f, 1.f));
    emit("v2", 0, seeded, v2); emit("v3", 0, seeded, v3); emit("color", 0, seeded, color);
    for (int index = 0; index < 3; ++index)
    {
      gz::msgs::Quaternion quaternion;
      gz::msgs::Pose pose;
      if (seeded)
      {
        quaternion.mutable_header()->mutable_stamp()->set_sec(11);
        pose.mutable_header()->mutable_stamp()->set_sec(11);
        pose.mutable_position()->mutable_header()->mutable_stamp()->set_sec(22);
        pose.mutable_orientation()->mutable_header()->mutable_stamp()->set_sec(33);
        pose.set_name("retained"); pose.set_id(42);
      }
      const auto &q = rotations[index];
      gz::msgs::Set(&quaternion, q);
      gz::msgs::Set(&pose, gz::math::Pose3d({1, 2, 3}, q));
      const auto reverse = gz::msgs::Convert(quaternion);
      const auto reversePose = gz::msgs::Convert(pose);
      if (Bits(reverse.X()) != Bits(q.X()) || Bits(reverse.Y()) != Bits(q.Y()) ||
          Bits(reverse.Z()) != Bits(q.Z()) || Bits(reverse.W()) != Bits(q.W()) ||
          Bits(reversePose.Rot().X()) != Bits(q.X()) || Bits(reversePose.Rot().Y()) != Bits(q.Y()) ||
          Bits(reversePose.Rot().Z()) != Bits(q.Z()) || Bits(reversePose.Rot().W()) != Bits(q.W())) return 1;
      emit("quaternion", index, seeded, quaternion); emit("pose", index, seeded, pose);
    }
  }
  std::cout << "]}\n";
}
