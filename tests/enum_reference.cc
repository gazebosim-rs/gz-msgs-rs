// Native reference only: use the actual pinned conversion policy headers.
#include <iostream>
#include <gz/msgs/convert/JointType.hh>
#include <gz/msgs/convert/GeometryType.hh>
#include <gz/msgs/convert/ShaderType.hh>
#include <gz/msgs/convert/PixelFormatType.hh>
#include <gz/msgs/convert/DiscoveryType.hh>
int main(int argc, char **argv) {
  if (argc != 4) return 2;
  const std::string family(argv[1]), mode(argv[2]), input(argv[3]);
#define FAMILY(name, type, function) \
  if (family == name) { \
    if (mode == "format") std::cout << gz::msgs::function(static_cast<type>(std::stoi(input))); \
    else std::cout << static_cast<int>(gz::msgs::function(input)); \
    return 0; \
  }
  FAMILY("Joint", gz::msgs::Joint::Type, ConvertJointType)
  FAMILY("Geometry", gz::msgs::Geometry::Type, ConvertGeometryType)
  FAMILY("Shader", gz::msgs::Material::ShaderType, ConvertShaderType)
  FAMILY("PixelFormat", gz::msgs::PixelFormatType, ConvertPixelFormatType)
  FAMILY("Discovery", gz::msgs::Discovery::Type, ConvertDiscoveryType)
  return 3;
}
