cmake_minimum_required(VERSION 3.12)
project(cpc_nvm3
  VERSION 0.1.0
  LANGUAGES C)

include(GNUInstallDirs)
find_package(PkgConfig REQUIRED)

option(DEBUG "Build the project using debug mode")


if(NOT DEFINED RUST_TARGET)
  set(RUST_TARGET host)
endif()

if(DEBUG)
  set(CARGO_PROFILE develop)
else()
  set(CARGO_PROFILE release)
endif()

message(STATUS "Build type: ${CARGO_PROFILE}")
set(PROFILE_DIR "target/${RUST_TARGET}/${CARGO_PROFILE}")

set(DOXYGEN_INPUT "${PROFILE_DIR}/cpc_nvm3.h")
configure_file(doc/cpc_nvm3.doxygen.in doc/cpc_nvm3.doxygen @ONLY)

add_custom_command(
  OUTPUT "${PROFILE_DIR}/libcpc_nvm3.so"
  COMMAND "${CMAKE_CURRENT_SOURCE_DIR}/do.sh" build -t "${RUST_TARGET}" -r "${CARGO_PROFILE}")

add_custom_command(
  OUTPUT "xml"
  COMMAND doxygen "doc/cpc_nvm3.doxygen")

add_custom_command(
  OUTPUT "api.md"
  COMMAND moxygen -g "./xml"
  DEPENDS "xml")

add_custom_target(cpc_nvm3_lib ALL
  DEPENDS "${PROFILE_DIR}/libcpc_nvm3.so")
set_target_properties(cpc_nvm3_lib
  PROPERTIES ADDITIONAL_CLEAN_FILES "${PROFILE_DIR}/build;${PROFILE_DIR}/deps;${PROFILE_DIR}/examples;${PROFILE_DIR}/incremental;${PROFILE_DIR}/cpc_nvm3.h;${PROFILE_DIR}/libcpc_nvm3.d")

add_custom_target(doc
  DEPENDS "api.md"
  COMMENT "Generating API documentation with Doxygen"
  VERBATIM)

add_library(cpc_nvm3 SHARED IMPORTED)
add_dependencies(cpc_nvm3 cpc_nvm3_lib)
set_target_properties(cpc_nvm3 PROPERTIES
  IMPORTED_LOCATION "${CMAKE_CURRENT_BINARY_DIR}/${PROFILE_DIR}/libcpc_nvm3.so")
target_include_directories(cpc_nvm3 INTERFACE "${CMAKE_CURRENT_BINARY_DIR}/${PROFILE_DIR}")

install(
  FILES "${CMAKE_CURRENT_BINARY_DIR}/${PROFILE_DIR}/libcpc_nvm3.so"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}"
  COMPONENT cpc_nvm3)

install(
  FILES "${CMAKE_CURRENT_BINARY_DIR}/${PROFILE_DIR}/cpc_nvm3.h"
  DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")

configure_file(libcpc_nvm3.pc.in libcpc_nvm3.pc @ONLY)
install(
  FILES "${CMAKE_CURRENT_BINARY_DIR}/libcpc_nvm3.pc"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig")
