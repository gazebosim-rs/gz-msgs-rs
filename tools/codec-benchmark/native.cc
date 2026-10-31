// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include "release_checks.hh"
#include <gz/msgs/clock.pb.h>
#include <gz/msgs/imu.pb.h>
#include <gz/msgs/laserscan.pb.h>
#include <gz/msgs/pose_v.pb.h>
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <stdexcept>
#include <string>

void header(gz::msgs::Header *h) {
  h->mutable_stamp()->set_sec(123); h->mutable_stamp()->set_nsec(456);
  auto *data = h->add_data(); data->set_key("frame_id"); data->add_value("robot/base");
}
void pose(gz::msgs::Pose *p, unsigned i) {
  header(p->mutable_header()); p->set_name("robot/" + std::to_string(i)); p->set_id(i + 1);
  auto *v = p->mutable_position(); v->set_x((i + 1) * 0.125); v->set_y(-0.25); v->set_z(-0.0);
  auto *q = p->mutable_orientation(); q->set_x(-0.0); q->set_z(0.5); q->set_w(1.0);
}
std::uint64_t sample(const gz::msgs::Clock &m) { return m.sim().nsec(); }
std::uint64_t sample(const gz::msgs::IMU &m) {
  auto value = m.angular_velocity().z(); std::uint64_t bits;
  std::memcpy(&bits, &value, sizeof(bits)); return bits;
}
std::uint64_t sample(const gz::msgs::Pose_V &m) { return m.pose_size() + m.pose(m.pose_size() - 1).id(); }
std::uint64_t sample(const gz::msgs::LaserScan &m) { return m.ranges_size() + m.count(); }
std::uint64_t encodedSample(const std::string &s) {
  return s.size() + static_cast<unsigned char>(s.front()) + static_cast<unsigned char>(s.back());
}
template<class T> void consume(const T &value) { asm volatile("" : : "g"(&value) : "memory"); }
void require(bool condition) { if (!condition) throw std::runtime_error("benchmark contract failed"); }
void report(std::uint64_t ns, std::uint64_t checksum, std::uint64_t count, std::size_t size) {
  std::cout << "{\"elapsed_ns\":" << ns << ",\"checksum\":" << checksum
            << ",\"iterations\":" << count << ",\"bytes\":" << size << "}\n";
}

template<class T> void run(const T &value, const std::string &action,
                          const std::string &mode, std::uint64_t count) {
  const auto bytes = value.SerializeAsString();
  { T check; require(check.ParseFromString(bytes)); require(check.SerializeAsString() == bytes); }
  if (action == "dump") { std::cout.write(bytes.data(), bytes.size()); return; }
  {
    T check; std::string encoded;
    for (unsigned i = 0; i < 2; ++i) {
      require(check.ParseFromString(bytes)); require(check.SerializeToString(&encoded)); require(encoded == bytes);
    }
  }
  T reused; std::string output; output.reserve(bytes.size());
  auto operation = [&]() {
    std::uint64_t result;
    if (mode == "encode_fresh") {
      auto encoded = value.SerializeAsString(); result = encodedSample(encoded); consume(encoded);
    } else if (mode == "encode_reuse") {
      require(value.SerializeToString(&output)); result = encodedSample(output); consume(output);
    } else if (mode == "decode_fresh") {
      T decoded; require(decoded.ParseFromString(bytes)); result = sample(decoded); consume(decoded);
    } else if (mode == "decode_reuse") {
      require(reused.ParseFromString(bytes)); result = sample(reused); consume(reused);
    } else if (mode == "roundtrip_fresh") {
      T decoded; require(decoded.ParseFromString(bytes)); auto encoded = decoded.SerializeAsString();
      result = sample(decoded) + encodedSample(encoded); consume(decoded); consume(encoded);
    } else if (mode == "roundtrip_reuse") {
      require(reused.ParseFromString(bytes)); require(reused.SerializeToString(&output));
      result = sample(reused) + encodedSample(output); consume(reused); consume(output);
    } else { throw std::runtime_error("unknown codec mode"); }
    return result;
  };
  if (action == "run") for (unsigned i = 0; i < 64; ++i) consume(operation());
  auto start = action == "run" ? std::chrono::steady_clock::now() : std::chrono::steady_clock::time_point{};
  std::uint64_t checksum = 0;
  for (std::uint64_t i = 0; i < count; ++i) checksum += operation();
  auto ns = action == "run" ? std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now() - start).count() : 0;
  if (mode == "decode_reuse" || mode == "roundtrip_reuse") require(reused.SerializeAsString() == bytes);
  if (mode == "encode_reuse" || mode == "roundtrip_reuse") require(output == bytes);
  report(ns, checksum, count, bytes.size());
}

void envelope(const std::string &fixture, const std::string &action, std::uint64_t count) {
  gz::msgs::Time value; value.set_sec(-1); value.set_nsec(42); auto bytes = value.SerializeAsString();
  if (fixture == "time_unknown") bytes.append("\xa0\x06\x7b", 3);
  else if (fixture == "time_wrong") bytes.append("\x09\0\0\0\0\0\0\0\0", 9);
  else require(fixture == "time_plain");
  gz::msgs::Time check; require(check.ParseFromString(bytes));
  require(check.sec() == -1 && check.nsec() == 42);
  if (action == "dump") { std::cout.write(bytes.data(), bytes.size()); return; }
  auto operation = [&]() {
    // Both envelopes own the original bytes and a fresh parsed typed value.
    auto original = bytes; gz::msgs::Time decoded; require(decoded.ParseFromString(original));
    auto result = decoded.nsec() + original.size(); consume(decoded); consume(original); return result;
  };
  if (action == "run") for (unsigned i = 0; i < 64; ++i) consume(operation());
  auto start = action == "run" ? std::chrono::steady_clock::now() : std::chrono::steady_clock::time_point{};
  std::uint64_t checksum = 0;
  for (std::uint64_t i = 0; i < count; ++i) checksum += operation();
  auto ns = action == "run" ? std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now() - start).count() : 0;
  report(ns, checksum, count, bytes.size());
}

int main(int argc, char **argv) {
  try {
    require(argc == 3 || argc == 5);
    const std::string action = argv[1], fixture = argv[2], mode = argc == 5 ? argv[3] : "";
    require(action == "dump" || action == "run" || action == "verify");
    if (argc == 5) { std::string n = argv[4]; require(!n.empty() && std::all_of(n.begin(), n.end(), [](char c) { return c >= '0' && c <= '9'; })); }
    auto count = argc == 5 ? std::stoull(argv[4]) : 0;
    require(action == "dump" || (count > 0 && count <= 1000000000));
    if (fixture.rfind("time_", 0) == 0) {
      require(action == "dump" || mode == "envelope"); envelope(fixture, action, count);
    } else if (fixture == "clock") {
      gz::msgs::Clock m; header(m.mutable_header());
      m.mutable_system()->set_sec(-1); m.mutable_system()->set_nsec(42);
      m.mutable_real()->set_sec(12); m.mutable_real()->set_nsec(34);
      m.mutable_sim()->set_sec(56); m.mutable_sim()->set_nsec(78); run(m, action, mode, count);
    } else if (fixture == "imu") {
      gz::msgs::IMU m; header(m.mutable_header()); m.set_entity_name("robot/imu");
      auto *q = m.mutable_orientation(); q->set_x(-0.0); q->set_z(0.5); q->set_w(1.0);
      for (auto *c : {m.mutable_orientation_covariance(), m.mutable_angular_velocity_covariance(), m.mutable_linear_acceleration_covariance()}) {
        header(c->mutable_header()); for (float v : {0.125f, -0.0f, 0.0f, 0.0f, 0.25f, 0.0f, 0.0f, 0.0f, 0.5f}) c->add_data(v);
      }
      auto *a = m.mutable_angular_velocity(); a->set_x(0.125); a->set_y(-0.25); a->set_z(0.5);
      auto *l = m.mutable_linear_acceleration(); l->set_x(-0.0); l->set_z(-9.75); run(m, action, mode, count);
    } else if (fixture.rfind("poses_", 0) == 0) {
      require(fixture == "poses_1" || fixture == "poses_100" || fixture == "poses_1000");
      auto n = std::stoul(fixture.substr(6)); gz::msgs::Pose_V m; header(m.mutable_header());
      for (unsigned i = 0; i < n; ++i) { pose(m.add_pose(), i); }
      run(m, action, mode, count);
    } else {
      require(fixture == "scan_360" || fixture == "scan_1080"); auto n = std::stoul(fixture.substr(5));
      gz::msgs::LaserScan m; header(m.mutable_header()); m.set_frame("robot/lidar"); pose(m.mutable_world_pose(), 0);
      m.set_angle_min(-1.0); m.set_angle_max(1.0); m.set_angle_step(0.015625); m.set_range_min(0.125); m.set_range_max(64.0);
      m.set_count(n); m.set_vertical_angle_min(-0.0); m.set_vertical_count(1);
      for (unsigned i = 0; i < n; ++i) { m.add_ranges(1.0 + (i % 128) * 0.125); m.add_intensities(i % 256); }
      run(m, action, mode, count);
    }
  } catch (const std::exception &error) { std::cerr << error.what() << '\n'; return 2; }
}
