cmake_minimum_required(VERSION 3.21)
project(qptp LANGUAGES CXX)
if(NOT WIN32 OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
  message(FATAL_ERROR "qptp requires 64-bit Windows")
endif()
add_library(driver_qptp SHARED src/driver.cpp src/debug_udp.cpp)
target_compile_features(driver_qptp PRIVATE cxx_std_17)
target_compile_definitions(driver_qptp PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
target_include_directories(driver_qptp PRIVATE vendor/openvr)
target_link_libraries(driver_qptp PRIVATE ws2_32)
set_target_properties(driver_qptp PROPERTIES
  RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/driver_qptp/bin/win64"
  RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_CURRENT_SOURCE_DIR}/driver_qptp/bin/win64"
  RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_CURRENT_SOURCE_DIR}/driver_qptp/bin/win64")
