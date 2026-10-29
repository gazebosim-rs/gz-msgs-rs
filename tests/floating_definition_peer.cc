// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Observe floating defaults and the pinned native declaration-number policy.
#include <google/protobuf/descriptor.h>
#include <google/protobuf/descriptor.pb.h>
#include <fstream>
#include <cstdint>
#include <cstring>
#include <google/protobuf/stubs/strutil.h>
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
  if (std::string(argv[1]) == "--bits") {
    std::string kind, text;
    while (std::cin >> kind >> text) {
      const uint64_t bits = std::stoull(text, nullptr, 16);
      if (kind == "f64") {
        double value; std::memcpy(&value, &bits, sizeof(value));
        std::cout << kind << ' ' << text << ' ' << google::protobuf::SimpleDtoa(value) << '\n';
      } else if (kind == "f32") {
        const uint32_t small = static_cast<uint32_t>(bits);
        float value; std::memcpy(&value, &small, sizeof(value));
        std::cout << kind << ' ' << text << ' ' << google::protobuf::SimpleFtoa(value) << '\n';
      } else return 9;
    }
    return std::cout.good() ? 0 : 8;
  }
  google::protobuf::FileDescriptorSet set;
  std::ifstream input(argv[1], std::ios::binary);
  if (!set.ParseFromIstream(&input)) return 3;
  google::protobuf::DescriptorPool pool;
  for (const auto &file : set.file())
    if (!pool.BuildFile(file)) return 5;
  const auto *message = pool.FindMessageTypeByName("Default");
  if (!message) return 6;
  const auto *field = message->FindFieldByName("value");
  if (!field) return 7;
  std::cout << "PROTOBUF " << GOOGLE_PROTOBUF_VERSION << '\n'
            << "HAS_DEFAULT " << field->has_default_value() << '\n';
  if (field->cpp_type() == google::protobuf::FieldDescriptor::CPPTYPE_FLOAT) {
    const float value = field->default_value_float(); uint32_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << "EFFECTIVE_F32_BITS " << bits << '\n';
  } else if (field->cpp_type() == google::protobuf::FieldDescriptor::CPPTYPE_DOUBLE) {
    const double value = field->default_value_double(); uint64_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << "EFFECTIVE_F64_BITS " << bits << '\n';
  } else return 7;
  std::cout << "DEFINITION_HEX " << Hex(message->DebugString()) << '\n';
  return std::cout.good() ? 0 : 8;
}
