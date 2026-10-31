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

#ifndef LGPROTOCOL_KVMFR_INPUT_H
#define LGPROTOCOL_KVMFR_INPUT_H

#include "KVMFRStream.h"

#define KVMFR_INPUT_VERSION                    3
#define KVMFR_INPUT_KEYBOARD_LEDS_VERSION      3
#define KVMFR_INPUT_STREAM_VERSION             1
#define KVMFR_INPUT_STREAM_ENDPOINT_COUNT      8
#define KVMFR_INPUT_STREAM_SLOT_COUNT          128
#define KVMFR_INPUT_STREAM_SLOT_SIZE           64
#define KVMFR_INPUT_MOUSE_BUTTON_COUNT         32
#define KVMFR_INPUT_MOUSE_ABSOLUTE_MAX         32767
#define KVMFR_INPUT_KEYBOARD_KEY_COUNT         6
#define KVMFR_INPUT_KEYBOARD_USAGE_MAX         231

typedef uint32_t KVMFRInputMessageType;

enum
{
  KVMFR_INPUT_MESSAGE_CLAIM          = 1,
  KVMFR_INPUT_MESSAGE_RELEASE        = 2,
  KVMFR_INPUT_MESSAGE_KEEPALIVE      = 3,
  KVMFR_INPUT_MESSAGE_RESET          = 4,
  KVMFR_INPUT_MESSAGE_MOUSE_RELATIVE = 5,
  KVMFR_INPUT_MESSAGE_MOUSE_ABSOLUTE = 6,
  KVMFR_INPUT_MESSAGE_KEYBOARD       = 7
};

typedef uint32_t KVMFRInputCapabilityFlags;

enum
{
  KVMFR_INPUT_CAP_MOUSE_RELATIVE = 1,
  KVMFR_INPUT_CAP_MOUSE_ABSOLUTE = 2,
  KVMFR_INPUT_CAP_KEYBOARD       = 4
};

typedef uint8_t KVMFRInputKeyboardLEDFlags;

enum
{
  KVMFR_INPUT_KEYBOARD_LED_NUM_LOCK    = 1,
  KVMFR_INPUT_KEYBOARD_LED_CAPS_LOCK   = 2,
  KVMFR_INPUT_KEYBOARD_LED_SCROLL_LOCK = 4,
  KVMFR_INPUT_KEYBOARD_LED_COMPOSE     = 8,
  KVMFR_INPUT_KEYBOARD_LED_KANA        = 16
};

typedef uint32_t KVMFRInputStatusFlags;

enum
{
  KVMFR_INPUT_STATUS_AVAILABLE           = 1,
  KVMFR_INPUT_STATUS_HAS_OWNER           = 2,
  KVMFR_INPUT_STATUS_KEYBOARD_LEDS_VALID = 4
};

typedef uint32_t KVMFRInputStreamEndpointFlags;

enum
{
  KVMFR_INPUT_STREAM_ENDPOINT_AVAILABLE = 1,
  KVMFR_INPUT_STREAM_ENDPOINT_BOUND     = 2
};

typedef uint32_t KVMFRInputMouseButtons;

typedef struct KVMFRInputMouseRelative
{
  KVMFRInputMouseButtons buttons;
  int32_t                deltaX;
  int32_t                deltaY;
  int32_t                wheel;
}
KVMFRInputMouseRelative;

typedef struct KVMFRInputMouseAbsolute
{
  KVMFRInputMouseButtons buttons;
  uint16_t               x;
  uint16_t               y;
  int32_t                wheel;
  uint32_t               reserved;
}
KVMFRInputMouseAbsolute;

typedef struct KVMFRInputKeyboard
{
  uint8_t modifiers;
  uint8_t keys[KVMFR_INPUT_KEYBOARD_KEY_COUNT];
  uint8_t reserved[9];
}
KVMFRInputKeyboard;

typedef union KVMFRInputPayload
{
  KVMFRInputMouseRelative mouseRelative;
  KVMFRInputMouseAbsolute mouseAbsolute;
  KVMFRInputKeyboard      keyboard;
  uint8_t                 reserved[16];
}
KVMFRInputPayload;

typedef struct KVMFRInputMessage
{
  KVMFRInputMessageType type;
  uint32_t              generation;
  uint32_t              sequence;
  uint32_t              reserved;
  KVMFRInputPayload     payload;
}
KVMFRInputMessage;

typedef struct KVMFRInputStreamEndpoint
{
  KVMFRStreamDescriptor          stream;
  uint32_t                       boundClientID;
  uint32_t                       bindingGeneration;
  KVMFRInputStreamEndpointFlags  flags;
  uint32_t                       reserved;
}
KVMFRInputStreamEndpoint;

typedef struct KVMFRInputStatus
{
  uint32_t                  version;
  KVMFRInputCapabilityFlags capabilities;
  KVMFRInputStatusFlags     flags;
  uint32_t                  generation;
  uint32_t                  ownerClientID;
  uint32_t                  ownerGeneration;
  uint32_t                  lease;
  uint32_t                  maxButtons;
  uint32_t                  streamVersion;
  uint32_t                  streamEndpointCount;
  uint32_t                  streamGeneration;
  uint8_t                   keyboardLEDs;
  uint8_t                   statusReserved[3];
  uint32_t                  streamReserved[4];
  KVMFRInputStreamEndpoint  streamEndpoint[KVMFR_INPUT_STREAM_ENDPOINT_COUNT];
}
KVMFRInputStatus;

#if defined(__cplusplus) && __cplusplus >= 201103L
#define LGPROTOCOL_INPUT_ASSERT(condition, message) static_assert(condition, message)
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
#define LGPROTOCOL_INPUT_ASSERT(condition, message) _Static_assert(condition, message)
#endif

#if defined(LGPROTOCOL_INPUT_ASSERT)
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputMouseRelative) == 16,
  "KVMFRInputMouseRelative size changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputMouseAbsolute) == 16,
  "KVMFRInputMouseAbsolute size changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputKeyboard) == 16,
  "KVMFRInputKeyboard size changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputPayload) == 16,
  "KVMFRInputPayload size changed");
LGPROTOCOL_INPUT_ASSERT(offsetof(KVMFRInputMessage, payload) == 16,
  "KVMFRInputMessage.payload offset changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputMessage) == 32,
  "KVMFRInputMessage size changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputStreamEndpoint) == 48,
  "KVMFRInputStreamEndpoint size changed");
LGPROTOCOL_INPUT_ASSERT(offsetof(KVMFRInputStatus, streamVersion) == 32,
  "KVMFRInputStatus.streamVersion offset changed");
LGPROTOCOL_INPUT_ASSERT(offsetof(KVMFRInputStatus, keyboardLEDs) == 44,
  "KVMFRInputStatus.keyboardLEDs offset changed");
LGPROTOCOL_INPUT_ASSERT(offsetof(KVMFRInputStatus, streamEndpoint) == 64,
  "KVMFRInputStatus.streamEndpoint offset changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputStatus) == 448,
  "KVMFRInputStatus size changed");
LGPROTOCOL_INPUT_ASSERT(sizeof(KVMFRInputMessage) <= KVMFR_INPUT_STREAM_SLOT_SIZE,
  "KVMFRInputMessage no longer fits a stream slot");
#undef LGPROTOCOL_INPUT_ASSERT
#endif

#endif
