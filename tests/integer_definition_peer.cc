// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Qualified declaration observer. BuildFile diagnostics remain on native stderr.
#include <google/protobuf/descriptor.h>
#include <google/protobuf/descriptor.pb.h>
#include <fstream>
#include <iostream>
#include <string>

static std::string Hex(const std::string &value)
{
  static constexpr char digits[] = "0123456789abcdef";
  std::string result;
  result.reserve(value.size() * 2);
  for (unsigned char byte : value)
  {
    result.push_back(digits[byte >> 4]);
    result.push_back(digits[byte & 15]);
  }
  return result;
}

int main(int argc, char **argv)
{
  if (argc != 2) return 2;
  std::ifstream input(argv[1], std::ios::binary);
  google::protobuf::FileDescriptorSet set;
  if (!set.ParseFromIstream(&input)) return 3;
  google::protobuf::DescriptorPool pool;
  for (const auto &file : set.file())
    if (!pool.BuildFile(file)) return 5;
  const auto *message = pool.FindMessageTypeByName("Default");
  if (!message) return 6;
  const auto *field = message->FindFieldByName("value");
  if (!field) return 7;
  std::cout << "PROTOBUF " << GOOGLE_PROTOBUF_VERSION << '\n';
  std::cout << "HAS_DEFAULT " << field->has_default_value() << '\n';
  google::protobuf::FieldDescriptorProto copied;
  field->CopyTo(&copied);
  std::cout << "COPY_DEFAULT_HEX " << Hex(copied.default_value()) << '\n';
  // This is the native effective value, separate from the retained raw input.
  using F = google::protobuf::FieldDescriptor;
  switch (field->cpp_type())
  {
    case F::CPPTYPE_INT32:
      std::cout << "EFFECTIVE_I32 " << field->default_value_int32() << '\n'; break;
    case F::CPPTYPE_INT64:
      std::cout << "EFFECTIVE_I64 " << field->default_value_int64() << '\n'; break;
    case F::CPPTYPE_UINT32:
      std::cout << "EFFECTIVE_U32 " << field->default_value_uint32() << '\n'; break;
    case F::CPPTYPE_UINT64:
      std::cout << "EFFECTIVE_U64 " << field->default_value_uint64() << '\n'; break;
    default:
      std::cout << "OTHER_CPP_TYPE " << field->cpp_type() << '\n'; break;
  }
  std::cout << "DEFINITION_HEX " << Hex(message->DebugString()) << '\n';
  return std::cout.good() ? 0 : 8;
}
