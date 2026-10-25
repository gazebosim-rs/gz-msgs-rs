// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <google/protobuf/descriptor.h>
#include <google/protobuf/descriptor.pb.h>
#include <google/protobuf/dynamic_message.h>
#include <climits>
#include <fstream>
#include <iostream>
#include <memory>
#include <string>
int main(int argc, char **argv) {
  if (argc != 2) return 2;
  if (std::string(argv[1]) == "--platform") {
    std::cout << GOOGLE_PROTOBUF_VERSION << ' ' << sizeof(long) * CHAR_BIT << '\n';
    return 0;
  }
  google::protobuf::FileDescriptorSet set;
  std::ifstream input(argv[1], std::ios::binary);
  if (!set.ParseFromIstream(&input)) return 3;
  google::protobuf::DescriptorPool pool;
  for (const auto &file : set.file()) if (!pool.BuildFile(file)) return 5;
  const auto *descriptor = pool.FindMessageTypeByName("Default");
  if (!descriptor) return 6;
  const auto *field = descriptor->FindFieldByName("value");
  if (!field) return 7;
  google::protobuf::DynamicMessageFactory factory;
  std::unique_ptr<google::protobuf::Message> message(factory.GetPrototype(descriptor)->New());
  const auto *reflection = message->GetReflection();
  using F = google::protobuf::FieldDescriptor;
  switch (field->cpp_type()) {
    case F::CPPTYPE_INT32:
      std::cout << field->default_value_int32();
      reflection->SetInt32(message.get(), field, field->default_value_int32()); break;
    case F::CPPTYPE_INT64:
      std::cout << field->default_value_int64();
      reflection->SetInt64(message.get(), field, field->default_value_int64()); break;
    case F::CPPTYPE_UINT32:
      std::cout << field->default_value_uint32();
      reflection->SetUInt32(message.get(), field, field->default_value_uint32()); break;
    case F::CPPTYPE_UINT64:
      std::cout << field->default_value_uint64();
      reflection->SetUInt64(message.get(), field, field->default_value_uint64()); break;
    default: return 8;
  }
  std::cout << '\n';
  const char *digits = "0123456789abcdef";
  for (unsigned char byte : message->SerializeAsString()) std::cout << digits[byte >> 4] << digits[byte & 15];
  std::cout << '\n';
}
