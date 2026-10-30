// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/Factory.hh>
#include <algorithm>
#include <cassert>
#include <cstdlib>
#include <filesystem>
#include <iostream>
#include <string>
#include <vector>
using Factory = gz::msgs::Factory;
void state(int step) {
  std::vector<std::string> names;
  Factory::Types(names);
  std::sort(names.begin(), names.end());
  std::cout << "STATE " << step;
  for (const auto &name : names)
    if (name.rfind("audit.", 0) == 0 && name != "audit.Registered")
      std::cout << ' ' << name;
  std::cout << '\n';
}
int main(int argc, char **argv) {
  if (argc != 5) return 2;
  const std::string mode = argv[1];
  const std::string other = argv[2];
  const std::string explicitPath = argv[3];
  const std::filesystem::path input = argv[4];
  if (mode == "generated") {
    assert(Factory::New("gz.msgs.StringMsg"));
    setenv("GZ_MSGS_INSTALL_PREFIX", other.c_str(), 1);
  }
  if (mode == "registered") {
    Factory::Register("audit.Registered", [] {
      return Factory::New("gz.msgs.StringMsg");
    });
    assert(Factory::New("audit.Registered"));
    setenv("GZ_MSGS_INSTALL_PREFIX", other.c_str(), 1);
  }
  if (mode == "late") setenv("GZ_MSGS_INSTALL_PREFIX", other.c_str(), 1);
  if (mode == "explicit-after" || mode == "errors")
    Factory::LoadDescriptors(explicitPath);
  state(1);
  if (mode == "freeze") setenv("GZ_MSGS_INSTALL_PREFIX", other.c_str(), 1);
  if (mode == "bad-share") {
    const auto share = std::filesystem::path(std::getenv("GZ_MSGS_INSTALL_PREFIX")) / "share/gz/protos";
    std::filesystem::copy_file(input / "A.desc", share / "a.desc", std::filesystem::copy_options::overwrite_existing);
    Factory::LoadDescriptors(share.string());
  }
  state(2);
  for (const auto &name : {"audit.A", "audit.B"}) {
    const auto message = Factory::New(name);
    std::cout << "MESSAGE " << name << ' ';
    if (message) std::cout << message->GetDescriptor()->file()->name();
    else std::cout << '-';
    std::cout << '\n';
  }
}
