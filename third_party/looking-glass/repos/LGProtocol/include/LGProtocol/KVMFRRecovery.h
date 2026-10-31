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

#ifndef LGPROTOCOL_KVMFR_RECOVERY_H
#define LGPROTOCOL_KVMFR_RECOVERY_H

#include <stddef.h>
#include <stdint.h>

#define KVMFR_R_MAGIC        "KVMFRRCV"
#define KVMFR_R_VERSION      1
#define KVMFR_R_READY        1
#define KVMFR_R_REGION_SIZE  65536u
#define KVMFR_R_LINE_SIZE    64u
#define KVMFR_R_HEARTBEAT_MS 250u
#define KVMFR_R_REQ_SLOTS    16

enum
{
  KVMFR_R_CAP_DISPLAY = 1
};

enum
{
  KVMFR_R_REQ_WRITING = 1,
  KVMFR_R_REQ_FIRST   = 2
};

enum
{
  KVMFR_R_REQ_NONE     = 0,
  KVMFR_R_REQ_NORMAL   = 1,
  KVMFR_R_REQ_RECOVERY = 2
};

enum
{
  KVMFR_R_STATE_UNKNOWN   = 0,
  KVMFR_R_STATE_NORMAL    = 1,
  KVMFR_R_STATE_SWITCHING = 2,
  KVMFR_R_STATE_ACTIVE    = 3,
  KVMFR_R_STATE_FAILED    = 4
};

enum
{
  KVMFR_R_ERR_NONE                = 0,
  KVMFR_R_ERR_UNSUPPORTED         = 1,
  KVMFR_R_ERR_HELPER_UNAVAILABLE  = 2,
  KVMFR_R_ERR_TOPOLOGY_FAILED     = 3,
  KVMFR_R_ERR_NO_FALLBACK_DISPLAY = 4,
  KVMFR_R_ERR_BUSY                = 5,
  KVMFR_R_ERR_CAPACITY            = 6
};

typedef struct KVMFRRHeader
{
  char     magic[8];
  uint16_t abiVersion;
  uint16_t structSize;
  uint32_t capabilities;
  uint32_t lgmpVersion;
  uint32_t kvmfrVersion;
  uint64_t session;
  uint8_t  uuid[16];
  uint32_t heartbeat;
  uint32_t reserved[2];
  uint32_t ready;
}
KVMFRRHeader;

typedef struct KVMFRRInfo
{
  char    version[48];
  uint8_t reserved[16];
}
KVMFRRInfo;

typedef struct KVMFRRReqHead
{
  uint32_t ticket;
  uint8_t  reserved[60];
}
KVMFRRReqHead;

typedef struct KVMFRRRequest
{
  uint32_t serial;
  uint32_t request;
  uint64_t session;
  uint8_t  reserved[48];
}
KVMFRRRequest;

typedef struct KVMFRRStatus
{
  uint32_t ackSerial;
  uint32_t ackRequest;
  uint32_t state;
  uint32_t error;
  uint64_t session;
  uint32_t serial;
  uint8_t  reserved[36];
}
KVMFRRStatus;

typedef struct KVMFRR
{
  KVMFRRHeader  header;
  KVMFRRInfo    info;
  KVMFRRReqHead req;
  KVMFRRRequest requests[KVMFR_R_REQ_SLOTS];
  KVMFRRStatus  status;
}
KVMFRR;

#if defined(__cplusplus) && __cplusplus >= 201103L
#define LGPROTOCOL_RECOVERY_ASSERT(condition, message) static_assert(condition, message)
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
#define LGPROTOCOL_RECOVERY_ASSERT(condition, message) _Static_assert(condition, message)
#endif

#if defined(LGPROTOCOL_RECOVERY_ASSERT)
LGPROTOCOL_RECOVERY_ASSERT(KVMFR_R_REGION_SIZE % KVMFR_R_LINE_SIZE == 0,
  "recovery region must contain whole cache lines");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRRHeader) == KVMFR_R_LINE_SIZE,
  "KVMFRRHeader size changed");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRRInfo) == KVMFR_R_LINE_SIZE,
  "KVMFRRInfo size changed");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRRReqHead) == KVMFR_R_LINE_SIZE,
  "KVMFRRReqHead size changed");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRRRequest) == KVMFR_R_LINE_SIZE,
  "KVMFRRRequest size changed");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRRStatus) == KVMFR_R_LINE_SIZE,
  "KVMFRRStatus size changed");

LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRR, header) == 0,
  "KVMFRR.header offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRR, info) == 64,
  "KVMFRR.info offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRR, req) == 128,
  "KVMFRR.req offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRR, requests) == 192,
  "KVMFRR.requests offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRR, status) == 1216,
  "KVMFRR.status offset changed");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRR) == 1280,
  "KVMFRR size changed");
LGPROTOCOL_RECOVERY_ASSERT(sizeof(KVMFRR) <= KVMFR_R_REGION_SIZE,
  "KVMFRR exceeds its recovery region");

LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRHeader, abiVersion) == 8,
  "KVMFRRHeader.abiVersion offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRHeader, capabilities) == 12,
  "KVMFRRHeader.capabilities offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRHeader, session) == 24,
  "KVMFRRHeader.session offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRHeader, uuid) == 32,
  "KVMFRRHeader.uuid offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRHeader, heartbeat) == 48,
  "KVMFRRHeader.heartbeat offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRHeader, ready) == 60,
  "KVMFRRHeader.ready offset changed");

LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRRequest, serial) == 0,
  "KVMFRRRequest.serial offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRRequest, request) == 4,
  "KVMFRRRequest.request offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRRequest, session) == 8,
  "KVMFRRRequest.session offset changed");

LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRStatus, ackSerial) == 0,
  "KVMFRRStatus.ackSerial offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRStatus, state) == 8,
  "KVMFRRStatus.state offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRStatus, session) == 16,
  "KVMFRRStatus.session offset changed");
LGPROTOCOL_RECOVERY_ASSERT(offsetof(KVMFRRStatus, serial) == 24,
  "KVMFRRStatus.serial offset changed");
#undef LGPROTOCOL_RECOVERY_ASSERT
#endif

#endif
