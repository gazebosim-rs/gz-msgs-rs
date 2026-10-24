// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <gz/msgs/Utility.hh>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <limits>
#include <sstream>
std::uint64_t Bits(double value) { std::uint64_t bits; std::memcpy(&bits,&value,8); return bits; }
std::string Hex(const std::string &bytes) { std::ostringstream out; for(unsigned char b:bytes) out<<std::hex<<std::setw(2)<<std::setfill('0')<<unsigned(b);return out.str(); }
std::string Wire(const google::protobuf::Message &message) {std::string bytes;if(!message.SerializeToString(&bytes))std::abort();return Hex(bytes);}
void Domain(const gz::math::SphericalCoordinates &value)
{
  std::cout<<"{\"surface\":"<<static_cast<int>(value.Surface())<<",\"bits\":["
    <<Bits(value.LatitudeReference().Radian())<<","<<Bits(value.LongitudeReference().Radian())<<","<<Bits(value.ElevationReference())<<","<<Bits(value.HeadingOffset().Radian())<<","<<Bits(value.SurfaceAxisEquatorial())<<","<<Bits(value.SurfaceAxisPolar())<<"]}";
}
template<class F> std::string Errors(F action) {std::ostringstream out;auto *old=std::cerr.rdbuf(out.rdbuf());action();std::cerr.rdbuf(old);return Hex(out.str());}
void Seed(gz::msgs::SphericalCoordinates &message)
{
  message.mutable_header()->mutable_stamp()->set_sec(11);
  message.mutable_entity()->set_id(42);message.mutable_entity()->set_name("retained");
  message.set_surface_model(gz::msgs::SphericalCoordinates::CUSTOM_SURFACE);
  message.set_surface_axis_equatorial(91);message.set_surface_axis_polar(92);
}
int main()
{
  using S=gz::math::SphericalCoordinates;
  std::cout<<"{\"from_math\":[";
  bool first=true;
  for(int index=0;index<8;++index)
  {
    S value;
    if(index==1||index==4)value.SetSurface(S::MOON_SCS);
    else if(index==2||index==3)value.SetSurface(S::CUSTOM_SURFACE,12000,10000);
    else if(index==5)value.SetSurface(S::CUSTOM_SURFACE,-1,1);
    else if(index==6)value.SetSurface(static_cast<S::SurfaceType>(500000));
    double latitude=1.1,longitude=2.2,elevation=3.3,heading=.4;
    if(index==3){latitude=1.9;longitude=2.8;elevation=3.7;heading=.6;}
    if(index==4){latitude=1.2;longitude=2.3;elevation=3.4;heading=.5;}
    if(index==7){latitude=-0.;longitude=-0.;elevation=-0.;heading=-0.;}
    value.SetLatitudeReference(GZ_DTOR(latitude));value.SetLongitudeReference(GZ_DTOR(longitude));value.SetElevationReference(elevation);value.SetHeadingOffset(GZ_DTOR(heading));
    for(int seeded=0;seeded<2;++seeded)
    {
      gz::msgs::SphericalCoordinates message;if(seeded)Seed(message);
      const auto errors=Errors([&]{gz::msgs::Set(&message,value);});
      if(!first)std::cout<<",";first=false;
      std::cout<<"{\"index\":"<<index<<",\"seeded\":"<<seeded<<",\"input\":";Domain(value);
      std::cout<<",\"wire_hex\":\""<<Wire(message)<<"\",\"stderr_hex\":\""<<errors<<"\"}";
    }
  }
  std::cout<<"],\"to_math\":[";first=true;
  for(int index=0;index<11;++index)
  {
    gz::msgs::SphericalCoordinates message;
    message.set_surface_model(static_cast<gz::msgs::SphericalCoordinates::SurfaceModel>(index==0||index==8||index==9?0:index==1?1:index==3?500000:2));
    double a=12000,b=10000;
    if(index==4){a=0;b=0;}if(index==5){a=1;b=2;}
    if(index==10){a=1;b=0;}
    if(index==6)a=std::numeric_limits<double>::quiet_NaN();
    if(index==7){a=std::numeric_limits<double>::infinity();b=a;}
    message.set_surface_axis_equatorial(a);message.set_surface_axis_polar(b);
    message.set_latitude_deg(index==8?-0.:index==9?1e308:1.1);
    message.set_longitude_deg(index==8?-0.:index==9?-1e308:2.2);
    message.set_elevation(index==8?-0.:3.3);message.set_heading_deg(index==8?-0.:index==9?1e308:.4);
    S value;const auto errors=Errors([&]{value=gz::msgs::Convert(message);});
    if(!first)std::cout<<",";first=false;
    std::cout<<"{\"index\":"<<index<<",\"wire_hex\":\""<<Wire(message)<<"\",\"output\":";Domain(value);
    std::cout<<",\"stderr_hex\":\""<<errors<<"\"}";
  }
  std::cout<<"]}\n";
}
