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

set(stage_dir    "${LGPROTOCOL_BINARY_DIR}/consumer-prefix")
set(consumer_dir "${LGPROTOCOL_BINARY_DIR}/consumer-build")

file(REMOVE_RECURSE "${stage_dir}" "${consumer_dir}")

execute_process(
  COMMAND "${CMAKE_COMMAND}" --install "${LGPROTOCOL_BINARY_DIR}"
    --prefix "${stage_dir}" --config "${LGPROTOCOL_TEST_CONFIG}"
  RESULT_VARIABLE install_result
)
if(NOT install_result EQUAL 0)
  message(FATAL_ERROR "LGProtocol staging install failed: ${install_result}")
endif()

set(expected_headers
  LGProtocol/KVMFR.h
  LGProtocol/KVMFRClipboard.h
  LGProtocol/KVMFRInput.h
  LGProtocol/KVMFRRecovery.h
  LGProtocol/KVMFRStream.h
  LGProtocol/KVMFRTypes.h
  LGProtocol/LGMPConfig.h
)
file(GLOB_RECURSE installed_headers
  LIST_DIRECTORIES FALSE
  RELATIVE "${stage_dir}/${LGPROTOCOL_INSTALL_INCLUDEDIR}"
  "${stage_dir}/${LGPROTOCOL_INSTALL_INCLUDEDIR}/*.h"
)
list(SORT installed_headers)
if(NOT "${installed_headers}" STREQUAL "${expected_headers}")
  message(FATAL_ERROR
    "Installed header manifest mismatch: ${installed_headers}"
  )
endif()

execute_process(
  COMMAND "${CMAKE_COMMAND}"
    -S "${LGPROTOCOL_SOURCE_DIR}/tests/package-consumer"
    -B "${consumer_dir}"
    -DCMAKE_PREFIX_PATH=${stage_dir}
    -DCMAKE_C_COMPILER=${LGPROTOCOL_TEST_COMPILER}
  RESULT_VARIABLE configure_result
)
if(NOT configure_result EQUAL 0)
  message(FATAL_ERROR
    "LGProtocol consumer configuration failed: ${configure_result}"
  )
endif()

execute_process(
  COMMAND "${CMAKE_COMMAND}" --build "${consumer_dir}"
    --config "${LGPROTOCOL_TEST_CONFIG}"
  RESULT_VARIABLE build_result
)
if(NOT build_result EQUAL 0)
  message(FATAL_ERROR "LGProtocol consumer build failed: ${build_result}")
endif()
