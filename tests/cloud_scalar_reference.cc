// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <iomanip>
#include <iostream>
#include <gz/msgs/PointCloudPackedUtils.hh>
using Cloud = gz::msgs::PointCloudPacked;
template<class T> void populate(Cloud &cloud) {
  gz::msgs::PointCloudPackedIterator<T> iter(cloud, "value");
  for (int i=0; iter != iter.End(); ++i, ++iter) *iter = static_cast<T>(i*3-6);
}
int main(int argc, char **argv) {
  if (argc!=3) return 2;
  Cloud cloud;
  const std::string mode=argv[1]; const int kind=std::stoi(argv[2]);
  if (mode=="scalar") {
    gz::msgs::InitPointCloudPacked(cloud,"scalar",true,{{"value",static_cast<Cloud::Field::DataType>(kind)}});
    cloud.mutable_data()->resize(5*cloud.point_step());
    switch(kind) {
      case 0: populate<int8_t>(cloud); break; case 1: populate<uint8_t>(cloud); break;
      case 2: populate<int16_t>(cloud); break; case 3: populate<uint16_t>(cloud); break;
      case 4: populate<int32_t>(cloud); break; case 5: populate<uint32_t>(cloud); break;
      case 6: populate<float>(cloud); break; case 7: populate<double>(cloud); break;
      default: return 2;
    }
  } else if (mode=="color") {
    gz::msgs::InitPointCloudPacked(cloud,"color",true,{{"xyz",Cloud::Field::FLOAT32},{"rgba",Cloud::Field::FLOAT32}});
    cloud.set_is_bigendian(kind!=0); cloud.mutable_data()->resize(5*cloud.point_step());
    gz::msgs::PointCloudPackedIterator<float> x(cloud,"x"), y(cloud,"y"), z(cloud,"z");
    gz::msgs::PointCloudPackedIterator<uint8_t> r(cloud,"r"), g(cloud,"g"), b(cloud,"b"), a(cloud,"a");
    for (int i=0;x!=x.End();++i,++x,++y,++z,++r,++g,++b,++a) {
      *x=i*2.0;*y=i*4.0;*z=i*6.0;*r=i*10;*g=i*20;*b=i*30;*a=i*40;
    }
  } else return 2;
  for (unsigned char byte:cloud.SerializeAsString()) std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<static_cast<int>(byte);
}
