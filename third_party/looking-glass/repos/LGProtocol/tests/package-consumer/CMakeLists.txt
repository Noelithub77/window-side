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

project(LGProtocolConsumer LANGUAGES C)

find_package(LGProtocol REQUIRED)

add_executable(lgprotocol_consumer main.c)
target_link_libraries(lgprotocol_consumer PRIVATE LGProtocol::LGProtocol)
set_target_properties(lgprotocol_consumer PROPERTIES
  C_STANDARD          11
  C_STANDARD_REQUIRED YES
  C_EXTENSIONS        NO
)
