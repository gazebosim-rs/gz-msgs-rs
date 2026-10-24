// Independent C++ protobuf oracle; descriptors come from system protoc.
#include <google/protobuf/descriptor.pb.h>
#include <google/protobuf/dynamic_message.h>
#include <google/protobuf/text_format.h>
#include <fstream>
#include <iostream>
#include <iterator>
#include <memory>
int main(int argc, char **argv) {
  if (argc != 4) return 2;
  google::protobuf::FileDescriptorSet set;
  std::ifstream descriptors(argv[1], std::ios::binary);
  if (!set.ParseFromIstream(&descriptors)) return 3;
  google::protobuf::DescriptorPool pool;
  for (const auto &file : set.file()) if (!pool.BuildFile(file)) return 4;
  auto descriptor = pool.FindMessageTypeByName(argv[2]);
  if (!descriptor) return 5;
  google::protobuf::DynamicMessageFactory factory(&pool);
  std::unique_ptr<google::protobuf::Message> message(factory.GetPrototype(descriptor)->New());
  std::string input(std::istreambuf_iterator<char>(std::cin), {});
  std::string mode(argv[3]);
  bool valid = mode == "encode" ? google::protobuf::TextFormat::ParseFromString(input, message.get()) : message->ParseFromString(input);
  if (!valid) return 6;
  if (mode == "decode") std::cout << message->DebugString();
  else if (!message->SerializeToOstream(&std::cout)) return 7;
}
