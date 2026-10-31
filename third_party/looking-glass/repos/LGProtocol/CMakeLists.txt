# LGProtocol - Looking Glass Communication Protocol
# Copyright © 2017-2026 Geoffrey McRae <geoff@hostfission.com>
# https://github.com/gnif/LGProtocol
# SPDX-License-Identifier: GPL-2.0-or-later
#
# This program is free software; you can redistribute it and/or modify it
# under the terms of the GNU General Public License as published by the Free
# Software Foundation; either version 2 of the License, or (at your option)
# any later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
# FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
# more details.
#
# You should have received a copy of the GNU General Public License along
# with this program; if not, write to the Free Software Foundation, Inc., 59
# Temple Place, Suite 330, Boston, MA 02111-1307 USA

cmake_minimum_required(VERSION 3.16)

project(LGProtocol VERSION 1.0.0 LANGUAGES C CXX)

include(CMakePackageConfigHelpers)
include(GNUInstallDirs)

add_library(LGProtocol INTERFACE)
add_library(LGProtocol::LGProtocol ALIAS LGProtocol)

target_compile_features(LGProtocol INTERFACE c_std_11 cxx_std_11)
target_include_directories(LGProtocol INTERFACE
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)

install(TARGETS LGProtocol
  EXPORT LGProtocolTargets
)
install(FILES
  include/LGProtocol/KVMFR.h
  include/LGProtocol/KVMFRClipboard.h
  include/LGProtocol/KVMFRInput.h
  include/LGProtocol/KVMFRRecovery.h
  include/LGProtocol/KVMFRStream.h
  include/LGProtocol/KVMFRTypes.h
  include/LGProtocol/LGMPConfig.h
  DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/LGProtocol
)
install(EXPORT LGProtocolTargets
  FILE LGProtocolTargets.cmake
  NAMESPACE LGProtocol::
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/LGProtocol
)

configure_package_config_file(
  cmake/LGProtocolConfig.cmake.in
  ${CMAKE_CURRENT_BINARY_DIR}/LGProtocolConfig.cmake
  INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/LGProtocol
)
write_basic_package_version_file(
  ${CMAKE_CURRENT_BINARY_DIR}/LGProtocolConfigVersion.cmake
  VERSION ${PROJECT_VERSION}
  COMPATIBILITY SameMajorVersion
)
install(FILES
  ${CMAKE_CURRENT_BINARY_DIR}/LGProtocolConfig.cmake
  ${CMAKE_CURRENT_BINARY_DIR}/LGProtocolConfigVersion.cmake
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/LGProtocol
)

include(CTest)

if(BUILD_TESTING)
  function(lgprotocol_enable_test_warnings target)
    if(MSVC)
      target_compile_options(${target} PRIVATE /W4 /WX)
    elseif(CMAKE_C_COMPILER_ID MATCHES "GNU|Clang" OR
           CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
      target_compile_options(${target} PRIVATE -Wall -Wextra -Werror)
    endif()
  endfunction()

  set(LGPROTOCOL_COMMON_HEADERS
    KVMFR
    KVMFRClipboard
    KVMFRInput
    KVMFRRecovery
    KVMFRStream
    LGMPConfig
    KVMFRTypes
  )

  set(header_id 0)
  foreach(header IN LISTS LGPROTOCOL_COMMON_HEADERS)
    math(EXPR header_id "${header_id} + 1")

    set(c_target lgprotocol_header_c_${header})
    add_executable(${c_target} tests/headers_c.c)
    target_compile_definitions(${c_target} PRIVATE
      LGPROTOCOL_HEADER_ID=${header_id}
    )
    target_link_libraries(${c_target} PRIVATE LGProtocol::LGProtocol)
    set_target_properties(${c_target} PROPERTIES
      C_STANDARD          11
      C_STANDARD_REQUIRED YES
      C_EXTENSIONS        NO
    )
    lgprotocol_enable_test_warnings(${c_target})
    add_test(NAME header_c_${header} COMMAND ${c_target})

    set(cxx_target lgprotocol_header_cxx_${header})
    add_executable(${cxx_target} tests/headers_cxx.cpp)
    target_compile_definitions(${cxx_target} PRIVATE
      LGPROTOCOL_HEADER_ID=${header_id}
    )
    target_link_libraries(${cxx_target} PRIVATE LGProtocol::LGProtocol)
    set_target_properties(${cxx_target} PROPERTIES
      CXX_STANDARD          11
      CXX_STANDARD_REQUIRED YES
      CXX_EXTENSIONS        YES
    )
    lgprotocol_enable_test_warnings(${cxx_target})
    add_test(NAME header_cxx_${header} COMMAND ${cxx_target})
  endforeach()

  add_executable(lgprotocol_abi_c tests/abi_common.c)
  target_link_libraries(lgprotocol_abi_c PRIVATE LGProtocol::LGProtocol)
  set_target_properties(lgprotocol_abi_c PROPERTIES
    C_STANDARD          11
    C_STANDARD_REQUIRED YES
    C_EXTENSIONS        NO
  )
  lgprotocol_enable_test_warnings(lgprotocol_abi_c)
  add_test(NAME abi_common_c COMMAND lgprotocol_abi_c)

  add_executable(lgprotocol_abi_cxx tests/abi_common.cpp)
  target_link_libraries(lgprotocol_abi_cxx PRIVATE LGProtocol::LGProtocol)
  set_target_properties(lgprotocol_abi_cxx PROPERTIES
    CXX_STANDARD          11
    CXX_STANDARD_REQUIRED YES
    CXX_EXTENSIONS        YES
  )
  lgprotocol_enable_test_warnings(lgprotocol_abi_cxx)
  add_test(NAME abi_common_cxx COMMAND lgprotocol_abi_cxx)

  add_test(NAME package_consumer
    COMMAND ${CMAKE_COMMAND}
      -DLGPROTOCOL_BINARY_DIR=${CMAKE_BINARY_DIR}
      -DLGPROTOCOL_SOURCE_DIR=${CMAKE_SOURCE_DIR}
      -DLGPROTOCOL_INSTALL_INCLUDEDIR=${CMAKE_INSTALL_INCLUDEDIR}
      -DLGPROTOCOL_TEST_COMPILER=${CMAKE_C_COMPILER}
      -DLGPROTOCOL_TEST_CONFIG=$<CONFIG>
      -P ${CMAKE_SOURCE_DIR}/tests/package-consumer/run.cmake
  )
endif()
