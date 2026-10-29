// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Observe byte defaults and declarations without changing the older observers.
#include <google/protobuf/descriptor.h>
#include <google/protobuf/descriptor.pb.h>
#include <fstream>
#include <iostream>
#include <string>

static std::string Hex(const std::string &value)
{
  static constexpr char digits[] = "0123456789abcdef";
  std::string result;
  for (unsigned char byte : value) {
    result.push_back(digits[byte >> 4]);
    result.push_back(digits[byte & 15]);
  }
  return result;
}
int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  google::protobuf::FileDescriptorSet set;
  std::ifstream input(argv[1], std::ios::binary);
  if (!set.ParseFromIstream(&input)) return 3;
  google::protobuf::DescriptorPool pool;
  for (const auto &file : set.file())
    if (!pool.BuildFile(file)) return 5;
  const auto *message = pool.FindMessageTypeByName("Default");
  if (!message) return 6;
  const auto *field = message->FindFieldByName("value");
  if (!field || field->cpp_type() != google::protobuf::FieldDescriptor::CPPTYPE_STRING)
    return 7;
  std::cout << "PROTOBUF " << GOOGLE_PROTOBUF_VERSION << '\n'
            << "HAS_DEFAULT " << field->has_default_value() << '\n'
            << "EFFECTIVE_BYTES_HEX " << Hex(field->default_value_string()) << '\n'
            << "DEFINITION_HEX " << Hex(message->DebugString()) << '\n';
  return std::cout.good() ? 0 : 8;
}
