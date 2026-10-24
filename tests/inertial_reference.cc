// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/Utility.hh>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>
std::uint64_t Bits(double value) { std::uint64_t bits; std::memcpy(&bits, &value, 8); return bits; }
std::string Hex(const google::protobuf::Message &message)
{
  std::string bytes;
  if (!message.SerializeToString(&bytes)) std::abort();
  std::ostringstream out;
  for (unsigned char b : bytes) out << std::hex << std::setw(2) << std::setfill('0') << unsigned(b);
  return out.str();
}
void Seed(gz::msgs::Inertial &message)
{
  message.mutable_header()->mutable_stamp()->set_sec(11);
  auto *pose = message.mutable_pose();
  pose->mutable_header()->mutable_stamp()->set_sec(22);
  pose->set_name("retained"); pose->set_id(42);
  pose->mutable_position()->mutable_header()->mutable_stamp()->set_sec(33);
  pose->mutable_position()->set_x(8); pose->mutable_position()->set_y(9); pose->mutable_position()->set_z(10);
  pose->mutable_orientation()->mutable_header()->mutable_stamp()->set_sec(44);
  pose->mutable_orientation()->set_x(.1); pose->mutable_orientation()->set_y(.2);
  pose->mutable_orientation()->set_z(.3); pose->mutable_orientation()->set_w(.4);
  message.add_fluid_added_mass(99);
}
int main()
{
  const gz::math::Pose3d pose(5, 6, 7, .4, .5, .6);
  std::cout << "{\"orientation_bits\":[" << Bits(pose.Rot().X()) << "," << Bits(pose.Rot().Y())
            << "," << Bits(pose.Rot().Z()) << "," << Bits(pose.Rot().W()) << "],\"rows\":[";
  bool first = true;
  for (int invalid = 0; invalid < 2; ++invalid)
    for (int kind = 0; kind < 4; ++kind)
      for (int seeded = 0; seeded < 2; ++seeded)
        for (int repeat = 0; repeat < (kind >= 2 ? 2 : 1); ++repeat)
        {
          const gz::math::MassMatrix3d mass = invalid ?
            gz::math::MassMatrix3d(-12, {-2, 0, 0}, {-.1, 2, 3}) :
            gz::math::MassMatrix3d(12, {2, 3, 4}, {.1, .2, .3});
          gz::math::Matrix6d added;
          int index = 1;
          for (int row = 0; row < 6; ++row)
            for (int col = row; col < 6; ++col)
            {
              const double value = index++ / 10.;
              added(row, col) = value;
              added(col, row) = kind == 3 && col != row ? -99 : value;
            }
          gz::msgs::Inertial message;
          if (seeded) Seed(message);
          if (kind == 0) gz::msgs::Set(&message, mass);
          else if (kind == 1) gz::msgs::Set(&message, gz::math::Inertiald(mass, pose));
          else for (int count = 0; count <= repeat; ++count)
            gz::msgs::Set(&message, gz::math::Inertiald(mass, pose, added));
          const auto reverse = gz::msgs::Convert(message);
          const auto &mm = reverse.MassMatrix();
          const auto &p = reverse.Pose();
          if (!first) std::cout << ",";
          first = false;
          std::cout << "{\"invalid\":" << invalid << ",\"kind\":" << kind << ",\"seeded\":" << seeded
                    << ",\"repeat\":" << repeat << ",\"wire_hex\":\"" << Hex(message) << "\",\"mass_bits\":["
                    << Bits(mm.Mass()) << "," << Bits(mm.Ixx()) << "," << Bits(mm.Iyy()) << "," << Bits(mm.Izz())
                    << "," << Bits(mm.Ixy()) << "," << Bits(mm.Ixz()) << "," << Bits(mm.Iyz()) << "],\"pose_bits\":["
                    << Bits(p.Pos().X()) << "," << Bits(p.Pos().Y()) << "," << Bits(p.Pos().Z()) << ","
                    << Bits(p.Rot().X()) << "," << Bits(p.Rot().Y()) << "," << Bits(p.Rot().Z()) << "," << Bits(p.Rot().W())
                    << "],\"added_bits\":";
          if (reverse.FluidAddedMass())
          {
            std::cout << "[";
            for (int row = 0; row < 6; ++row) for (int col = 0; col < 6; ++col)
            { if (row || col) std::cout << ","; std::cout << Bits(reverse.FluidAddedMass().value()(row,col)); }
            std::cout << "]";
          }
          else std::cout << "null";
          std::cout << "}";
        }
  std::cout << "]}\n";
}
