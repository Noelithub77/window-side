/**
 * LGProtocol - Looking Glass Communication Protocol
 * Copyright © 2017-2026 Geoffrey McRae <geoff@hostfission.com>
 * https://github.com/gnif/LGProtocol
 * SPDX-License-Identifier: GPL-2.0-or-later
 *
 * This program is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License as published by the Free
 * Software Foundation; either version 2 of the License, or (at your option)
 * any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc., 59
 * Temple Place, Suite 330, Boston, MA 02111-1307 USA
 */

#ifndef LGPROTOCOL_KVMFR_STREAM_H
#define LGPROTOCOL_KVMFR_STREAM_H

#include <stddef.h>
#include <stdint.h>

typedef struct KVMFRStreamDescriptor
{
  uint32_t magic;
  uint16_t version;
  uint16_t size;
  uint32_t offset;
  uint32_t regionSize;
  uint32_t direction;
  uint32_t policy;
  uint32_t slotCount;
  uint32_t slotSize;
}
KVMFRStreamDescriptor;

#if defined(__cplusplus) && __cplusplus >= 201103L
#define LGPROTOCOL_STREAM_ASSERT(condition, message) static_assert(condition, message)
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
#define LGPROTOCOL_STREAM_ASSERT(condition, message) _Static_assert(condition, message)
#endif

#if defined(LGPROTOCOL_STREAM_ASSERT)
LGPROTOCOL_STREAM_ASSERT(offsetof(KVMFRStreamDescriptor, magic) == 0,
  "KVMFRStreamDescriptor.magic offset changed");
LGPROTOCOL_STREAM_ASSERT(offsetof(KVMFRStreamDescriptor, offset) == 8,
  "KVMFRStreamDescriptor.offset offset changed");
LGPROTOCOL_STREAM_ASSERT(offsetof(KVMFRStreamDescriptor, slotSize) == 28,
  "KVMFRStreamDescriptor.slotSize offset changed");
LGPROTOCOL_STREAM_ASSERT(sizeof(KVMFRStreamDescriptor) == 32,
  "KVMFRStreamDescriptor size changed");
#undef LGPROTOCOL_STREAM_ASSERT
#endif

#endif
