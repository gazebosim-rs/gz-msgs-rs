// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#include <algorithm>
#include <iostream>
#include <gz/msgs/MessageFactory.hh>
#include <gz/msgs/stringmsg.pb.h>
int main() {
  gz::msgs::MessageFactory factory;
  factory.Register("custom",[]{auto value=std::make_unique<gz::msgs::StringMsg>();value->set_data("first");return value;});
  const bool first=static_cast<gz::msgs::StringMsg *>(factory.New("custom").get())->data()=="first";
  factory.Register("custom",[]{auto value=std::make_unique<gz::msgs::StringMsg>();value->set_data("second");return value;});
  auto parsed=factory.New("custom","");
  const bool cleared=parsed && static_cast<gz::msgs::StringMsg *>(parsed.get())->data().empty();
  factory.Register("reentrant",[&factory]{return factory.New("custom");});
  auto reentrant=factory.New("reentrant");
  const bool created=reentrant && static_cast<gz::msgs::StringMsg *>(reentrant.get())->data()=="second";
  factory.Register(".registered",[]{return std::make_unique<gz::msgs::StringMsg>();});
  const bool dotted=static_cast<bool>(factory.New(".registered"));
  bool rejected=true;
  for(const auto &name:{"StringMsg","example.msgs.StringMsg",".google.protobuf.Any","..gz.msgs.StringMsg"}) rejected=rejected && !factory.New(name);
  std::vector<std::string> names;factory.Types(names);std::sort(names.begin(),names.end());
  const bool listed=std::find(names.begin(),names.end(),"custom")!=names.end() && std::find(names.begin(),names.end(),"reentrant")!=names.end();
  if (!first || !cleared || !created || !listed || !dotted || !rejected) return 2;
  std::cout<<"{\"first_constructor\":true,\"text_clears_constructor_fields\":true,\"reentrant_created\":true,\"constructors_listed\":true,\"literal_dot_constructor\":true,\"unrecognized_names_rejected\":true,\"types_count\":"<<names.size()<<"}";
}
