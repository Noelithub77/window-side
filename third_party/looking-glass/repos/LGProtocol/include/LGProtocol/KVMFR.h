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

#ifndef LGPROTOCOL_KVMFR_H
#define LGPROTOCOL_KVMFR_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#if defined(__cplusplus)
#include <atomic>
#else
#include <stdatomic.h>
#endif

#include "KVMFRTypes.h"
#include "LGMPConfig.h"
#include "KVMFRClipboard.h"
#include "KVMFRInput.h"

#define KVMFR_MAGIC                   "KVMFR---"
#define KVMFR_VERSION                 34
#define KVMFR_SDR_WHITE_LEVEL_DEFAULT 203
#define KVMFR_MAX_DAMAGE_RECTS        64

typedef uint32_t KVMFRCursorFlags;

enum
{
  CURSOR_FLAG_POSITION        = 1,
  CURSOR_FLAG_VISIBLE         = 2,
  CURSOR_FLAG_SHAPE           = 4,
  CURSOR_FLAG_COLOR_TRANSFORM = 8,
  CURSOR_FLAG_VISIBLE_VALID   = 16
};

typedef uint32_t KVMFRFeatureFlags;

enum
{
  KVMFR_FEATURE_SETCURSORPOS   = 1,
  KVMFR_FEATURE_WINDOWSIZE     = 2,
  KVMFR_FEATURE_FRAME_SCHEDULE = 4,
  KVMFR_FEATURE_INPUT          = 8,
  KVMFR_FEATURE_CLIPBOARD      = 16
};

typedef uint32_t KVMFRMessageType;

enum
{
  KVMFR_MESSAGE_SETCURSORPOS   = 0,
  KVMFR_MESSAGE_WINDOWSIZE     = 1,
  KVMFR_MESSAGE_FRAME_SCHEDULE = 2
};

enum
{
  KVMFR_RECORD_VMINFO = 1,
  KVMFR_RECORD_OSINFO = 2
};

typedef enum KVMFROS
{
  KVMFR_OS_LINUX   = 0,
  KVMFR_OS_BSD     = 1,
  KVMFR_OS_OSX     = 2,
  KVMFR_OS_WINDOWS = 3,
  KVMFR_OS_OTHER   = 4
}
KVMFROS;

enum
{
  KVMFR_COLOR_TRANSFORM_MATRIX = 1,
  KVMFR_COLOR_TRANSFORM_LUT    = 2
};

typedef uint32_t KVMFRFrameFlags;

enum
{
  FRAME_FLAG_BLOCK_SCREENSAVER   = 1,
  FRAME_FLAG_REQUEST_ACTIVATION  = 2,
  FRAME_FLAG_TRUNCATED           = 4,
  FRAME_FLAG_HDR                 = 8,
  FRAME_FLAG_HDR_PQ              = 16,
  FRAME_FLAG_HDR_METADATA        = 32
};

typedef uint32_t KVMFRFrameTimingFlags;

enum
{
  KVMFR_FRAME_TIMING_PHASE_VALID = 1
};

typedef uint32_t KVMFRFrameScheduleFlags;

enum
{
  KVMFR_FRAME_SCHEDULE_ACTIVE    = 1,
  KVMFR_FRAME_SCHEDULE_RELEASE   = 2,
  KVMFR_FRAME_SCHEDULE_RESET     = 4,
  KVMFR_FRAME_SCHEDULE_IMMEDIATE = 8
};

typedef struct KVMFR
{
  char              magic[8];
  uint32_t          version;
  char              hostver[32];
  KVMFRFeatureFlags features;
}
KVMFR;

#if defined(_MSC_VER)
#pragma warning(push)
#pragma warning(disable: 4200)
#endif

typedef struct KVMFRRecord
{
  uint8_t  type;
  uint32_t size;
  uint8_t  data[];
}
KVMFRRecord;

typedef struct KVMFRRecord_VMInfo
{
  uint8_t uuid[16];
  char    capture[32];
  uint8_t cpus;
  uint8_t cores;
  uint8_t sockets;
  char    model[];
}
KVMFRRecord_VMInfo;

typedef struct KVMFRRecord_OSInfo
{
  uint8_t os;
  char    name[];
}
KVMFRRecord_OSInfo;

typedef struct KVMFRCursor
{
  int16_t         x;
  int16_t         y;
  KVMFRCursorType type;
  int8_t          hx;
  int8_t          hy;
  uint32_t        width;
  uint32_t        height;
  uint32_t        pitch;
  uint32_t        sdrWhiteLevel;
}
KVMFRCursor;

#if defined(__cplusplus)
typedef std::atomic_uint_least32_t KVMFRFrameBufferWritePointer;
#else
typedef atomic_uint_least32_t KVMFRFrameBufferWritePointer;
#endif

#define KVMFR_FRAMEBUFFER_WP_SIZE 4u

typedef struct KVMFRFrameBuffer
{
  KVMFRFrameBufferWritePointer wp;
  uint8_t                      data[0];
}
KVMFRFrameBuffer;

typedef struct KVMFRFrame
{
  uint32_t              formatVer;
  uint32_t              frameSerial;
  KVMFRFrameType        type;
  uint32_t              screenWidth;
  uint32_t              screenHeight;
  uint32_t              dataWidth;
  uint32_t              dataHeight;
  uint32_t              frameWidth;
  uint32_t              frameHeight;
  KVMFRFrameRotation    rotation;
  uint32_t              stride;
  uint32_t              pitch;
  uint32_t              offset;
  KVMFRFrameFlags       flags;
  uint32_t              damageRectsCount;
  uint32_t              sdrWhiteLevel;
  uint64_t              captureTime;
  uint64_t              postProcessTime;
  uint64_t              copyTime;
  uint64_t              readyTime;
  uint64_t              holdTime;
  uint64_t              readyLeadTime;
  uint32_t              timingSerial;
  uint32_t              timingValid;
  KVMFRFrameTimingFlags timingFlags;
  uint8_t               timingReserved[4];
  uint16_t              hdrDisplayPrimary[3][2];
  uint16_t              hdrWhitePoint[2];
  uint32_t              hdrMaxDisplayLuminance;
  uint32_t              hdrMinDisplayLuminance;
  uint32_t              hdrMaxContentLightLevel;
  uint32_t              hdrMaxFrameAverageLightLevel;
  uint32_t              scheduleGeneration;
  uint32_t              scheduleEpoch;
  uint32_t              scheduleDeadlineSerial;
  uint8_t               hdrReserved[20];
  KVMFRFrameDamageRect  damageRects[KVMFR_MAX_DAMAGE_RECTS];
}
KVMFRFrame;

#if defined(_MSC_VER)
#pragma warning(pop)
#endif

typedef struct KVMFRMessage
{
  KVMFRMessageType type;
}
KVMFRMessage;

typedef struct KVMFRSetCursorPos
{
  KVMFRMessage msg;
  int32_t      x;
  int32_t      y;
}
KVMFRSetCursorPos;

typedef struct KVMFRWindowSize
{
  KVMFRMessage msg;
  uint32_t     w;
  uint32_t     h;
}
KVMFRWindowSize;

typedef struct KVMFRFrameSchedule
{
  KVMFRMessage            msg;
  uint32_t                clientID;
  uint32_t                generation;
  KVMFRFrameScheduleFlags flags;
  uint64_t                period;
  uint64_t                targetSlack;
  int64_t                 phaseError;
  uint32_t                feedbackFrameSerial;
  uint32_t                feedbackScheduleEpoch;
  uint32_t                feedbackDeadlineSerial;
  uint32_t                lease;
  uint8_t                 reserved[8];
}
KVMFRFrameSchedule;

#if defined(__cplusplus) && __cplusplus >= 201103L
#define LGPROTOCOL_KVMFR_ASSERT(condition, message) static_assert(condition, message)
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
#define LGPROTOCOL_KVMFR_ASSERT(condition, message) _Static_assert(condition, message)
#endif

#if defined(LGPROTOCOL_KVMFR_ASSERT)
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, captureTime) == 64,
  "KVMFRFrame.captureTime offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, holdTime) == 96,
  "KVMFRFrame.holdTime offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, readyLeadTime) == 104,
  "KVMFRFrame.readyLeadTime offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, timingSerial) == 112,
  "KVMFRFrame.timingSerial offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, timingFlags) == 120,
  "KVMFRFrame.timingFlags offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, hdrDisplayPrimary) == 128,
  "KVMFRFrame.hdrDisplayPrimary offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, scheduleDeadlineSerial) == 168,
  "KVMFRFrame.scheduleDeadlineSerial offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrame, damageRects) == 192,
  "KVMFRFrame.damageRects offset changed");

LGPROTOCOL_KVMFR_ASSERT(
  offsetof(KVMFRFrameSchedule, feedbackDeadlineSerial) == 48,
  "KVMFRFrameSchedule.feedbackDeadlineSerial offset changed");
LGPROTOCOL_KVMFR_ASSERT(offsetof(KVMFRFrameSchedule, lease) == 52,
  "KVMFRFrameSchedule.lease offset changed");
LGPROTOCOL_KVMFR_ASSERT(sizeof(KVMFRFrameSchedule) == 64,
  "KVMFRFrameSchedule size changed");
#undef LGPROTOCOL_KVMFR_ASSERT
#endif

#endif
