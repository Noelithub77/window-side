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

#include <LGProtocol/KVMFR.h>
#include <LGProtocol/KVMFRRecovery.h>

#include <float.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#if defined(__cplusplus)
#include <type_traits>
#define ABI_ASSERT(condition) static_assert((condition), #condition)
#define ABI_ALIGNOF(type)     alignof(type)
#else
#define ABI_ASSERT(condition) _Static_assert((condition), #condition)
#define ABI_ALIGNOF(type)     _Alignof(type)
#endif

#define CHECK_CONST(identifier, expected)   \
  ABI_ASSERT((identifier) == (expected))
#define CHECK_SIZE(type, expected)          \
  ABI_ASSERT(sizeof(type) == (expected))
#define CHECK_ALIGN(type, expected)         \
  ABI_ASSERT(ABI_ALIGNOF(type) == (expected))
#define CHECK_OFFSET(type, field, expected) \
  ABI_ASSERT(offsetof(type, field) == (expected))
#define CHECK_ARRAY(type, field, expected)  \
  ABI_ASSERT(sizeof(((type *)0)->field) / sizeof(((type *)0)->field[0]) == \
    (expected))

ABI_ASSERT(CHAR_BIT == 8);
ABI_ASSERT(sizeof(int) == 4);
ABI_ASSERT(sizeof(float) == 4);
ABI_ASSERT(FLT_RADIX == 2);
ABI_ASSERT(FLT_MANT_DIG == 24);
ABI_ASSERT(FLT_MAX_EXP == 128);
ABI_ASSERT(FLT_MIN_EXP == -125);
ABI_ASSERT(UINT_LEAST32_MAX == UINT32_MAX);

#if defined(__BYTE_ORDER__) && defined(__ORDER_LITTLE_ENDIAN__)
ABI_ASSERT(__BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__);
#endif

CHECK_CONST(LGMP_Q_POINTER, 1);
CHECK_CONST(LGMP_Q_FRAME, 2);
CHECK_CONST(LGMP_Q_FRAME_OWNER, 3);
CHECK_CONST(LGMP_Q_INPUT, 5);
CHECK_CONST(LGMP_Q_CLIPBOARD, 6);
CHECK_CONST(LGMP_Q_FRAME_LEN, 2);
CHECK_CONST(LGMP_Q_FRAME_BUFFER_LEN, 3);
CHECK_CONST(LGMP_Q_POINTER_LEN, 32);
CHECK_CONST(LGMP_Q_INPUT_LEN, 4);
CHECK_CONST(LGMP_Q_CLIPBOARD_LEN, 32);

CHECK_CONST(FRAME_TYPE_INVALID, 0);
CHECK_CONST(FRAME_TYPE_BGRA, 1);
CHECK_CONST(FRAME_TYPE_RGBA, 2);
CHECK_CONST(FRAME_TYPE_RGBA10, 3);
CHECK_CONST(FRAME_TYPE_RGBA16F, 4);
CHECK_CONST(FRAME_TYPE_BGR_32, 5);
CHECK_CONST(FRAME_TYPE_RGB_24, 6);
CHECK_CONST(FRAME_TYPE_MAX, 7);
CHECK_CONST(FRAME_ROT_0, 0);
CHECK_CONST(FRAME_ROT_90, 1);
CHECK_CONST(FRAME_ROT_180, 2);
CHECK_CONST(FRAME_ROT_270, 3);
CHECK_CONST(CURSOR_TYPE_COLOR, 0);
CHECK_CONST(CURSOR_TYPE_MONOCHROME, 1);
CHECK_CONST(CURSOR_TYPE_MASKED_COLOR, 2);
CHECK_CONST(LG_COLOR_TRANSFORM_MATRIX, 1);
CHECK_CONST(LG_COLOR_TRANSFORM_LUT, 2);
CHECK_CONST(LG_SDR_WHITE_LEVEL_DEFAULT, 203);
CHECK_CONST(LG_MAX_FRAME_DAMAGE_RECTS, 64);

CHECK_SIZE(KVMFRFrameType, 4);
CHECK_SIZE(KVMFRFrameRotation, 4);
CHECK_SIZE(KVMFRCursorType, 4);
CHECK_SIZE(KVMFRColorTransformFlags, 4);
CHECK_SIZE(KVMFRFrameDamageRect, 16);
CHECK_ALIGN(KVMFRFrameDamageRect, 4);
CHECK_OFFSET(KVMFRFrameDamageRect, x, 0);
CHECK_OFFSET(KVMFRFrameDamageRect, y, 4);
CHECK_OFFSET(KVMFRFrameDamageRect, width, 8);
CHECK_OFFSET(KVMFRFrameDamageRect, height, 12);
CHECK_SIZE(KVMFRColorTransform, 65592);
CHECK_ALIGN(KVMFRColorTransform, 4);
CHECK_OFFSET(KVMFRColorTransform, flags, 0);
CHECK_OFFSET(KVMFRColorTransform, matrix, 4);
CHECK_OFFSET(KVMFRColorTransform, scalar, 52);
CHECK_OFFSET(KVMFRColorTransform, lut, 56);
CHECK_ARRAY(KVMFRColorTransform, matrix, 3);
ABI_ASSERT(sizeof(((KVMFRColorTransform *)0)->matrix[0]) / sizeof(float) == 4);
CHECK_ARRAY(KVMFRColorTransform, lut, 4096);
ABI_ASSERT(sizeof(((KVMFRColorTransform *)0)->lut[0]) / sizeof(float) == 4);

CHECK_SIZE(KVMFRStreamDescriptor, 32);
CHECK_ALIGN(KVMFRStreamDescriptor, 4);
CHECK_OFFSET(KVMFRStreamDescriptor, magic, 0);
CHECK_OFFSET(KVMFRStreamDescriptor, version, 4);
CHECK_OFFSET(KVMFRStreamDescriptor, size, 6);
CHECK_OFFSET(KVMFRStreamDescriptor, offset, 8);
CHECK_OFFSET(KVMFRStreamDescriptor, regionSize, 12);
CHECK_OFFSET(KVMFRStreamDescriptor, direction, 16);
CHECK_OFFSET(KVMFRStreamDescriptor, policy, 20);
CHECK_OFFSET(KVMFRStreamDescriptor, slotCount, 24);
CHECK_OFFSET(KVMFRStreamDescriptor, slotSize, 28);

CHECK_CONST(KVMFR_INPUT_VERSION, 3);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_LEDS_VERSION, 3);
CHECK_CONST(KVMFR_INPUT_STREAM_VERSION, 1);
CHECK_CONST(KVMFR_INPUT_STREAM_ENDPOINT_COUNT, 8);
CHECK_CONST(KVMFR_INPUT_STREAM_SLOT_COUNT, 128);
CHECK_CONST(KVMFR_INPUT_STREAM_SLOT_SIZE, 64);
CHECK_CONST(KVMFR_INPUT_MOUSE_BUTTON_COUNT, 32);
CHECK_CONST(KVMFR_INPUT_MOUSE_ABSOLUTE_MAX, 32767);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_KEY_COUNT, 6);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_USAGE_MAX, 231);
CHECK_CONST(KVMFR_INPUT_MESSAGE_CLAIM, 1);
CHECK_CONST(KVMFR_INPUT_MESSAGE_RELEASE, 2);
CHECK_CONST(KVMFR_INPUT_MESSAGE_KEEPALIVE, 3);
CHECK_CONST(KVMFR_INPUT_MESSAGE_RESET, 4);
CHECK_CONST(KVMFR_INPUT_MESSAGE_MOUSE_RELATIVE, 5);
CHECK_CONST(KVMFR_INPUT_MESSAGE_MOUSE_ABSOLUTE, 6);
CHECK_CONST(KVMFR_INPUT_MESSAGE_KEYBOARD, 7);
CHECK_CONST(KVMFR_INPUT_CAP_MOUSE_RELATIVE, 1);
CHECK_CONST(KVMFR_INPUT_CAP_MOUSE_ABSOLUTE, 2);
CHECK_CONST(KVMFR_INPUT_CAP_KEYBOARD, 4);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_LED_NUM_LOCK, 1);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_LED_CAPS_LOCK, 2);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_LED_SCROLL_LOCK, 4);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_LED_COMPOSE, 8);
CHECK_CONST(KVMFR_INPUT_KEYBOARD_LED_KANA, 16);
CHECK_CONST(KVMFR_INPUT_STATUS_AVAILABLE, 1);
CHECK_CONST(KVMFR_INPUT_STATUS_HAS_OWNER, 2);
CHECK_CONST(KVMFR_INPUT_STATUS_KEYBOARD_LEDS_VALID, 4);
CHECK_CONST(KVMFR_INPUT_STREAM_ENDPOINT_AVAILABLE, 1);
CHECK_CONST(KVMFR_INPUT_STREAM_ENDPOINT_BOUND, 2);

CHECK_SIZE(KVMFRInputMessageType, 4);
CHECK_SIZE(KVMFRInputCapabilityFlags, 4);
CHECK_SIZE(KVMFRInputKeyboardLEDFlags, 1);
CHECK_SIZE(KVMFRInputStatusFlags, 4);
CHECK_SIZE(KVMFRInputStreamEndpointFlags, 4);
CHECK_SIZE(KVMFRInputMouseButtons, 4);

CHECK_SIZE(KVMFRInputMouseRelative, 16);
CHECK_ALIGN(KVMFRInputMouseRelative, 4);
CHECK_OFFSET(KVMFRInputMouseRelative, buttons, 0);
CHECK_OFFSET(KVMFRInputMouseRelative, deltaX, 4);
CHECK_OFFSET(KVMFRInputMouseRelative, deltaY, 8);
CHECK_OFFSET(KVMFRInputMouseRelative, wheel, 12);

CHECK_SIZE(KVMFRInputMouseAbsolute, 16);
CHECK_ALIGN(KVMFRInputMouseAbsolute, 4);
CHECK_OFFSET(KVMFRInputMouseAbsolute, buttons, 0);
CHECK_OFFSET(KVMFRInputMouseAbsolute, x, 4);
CHECK_OFFSET(KVMFRInputMouseAbsolute, y, 6);
CHECK_OFFSET(KVMFRInputMouseAbsolute, wheel, 8);
CHECK_OFFSET(KVMFRInputMouseAbsolute, reserved, 12);

CHECK_SIZE(KVMFRInputKeyboard, 16);
CHECK_ALIGN(KVMFRInputKeyboard, 1);
CHECK_OFFSET(KVMFRInputKeyboard, modifiers, 0);
CHECK_OFFSET(KVMFRInputKeyboard, keys, 1);
CHECK_OFFSET(KVMFRInputKeyboard, reserved, 7);
CHECK_ARRAY(KVMFRInputKeyboard, keys, 6);
CHECK_ARRAY(KVMFRInputKeyboard, reserved, 9);

CHECK_SIZE(KVMFRInputPayload, 16);
CHECK_ALIGN(KVMFRInputPayload, 4);
CHECK_OFFSET(KVMFRInputPayload, mouseRelative, 0);
CHECK_OFFSET(KVMFRInputPayload, mouseAbsolute, 0);
CHECK_OFFSET(KVMFRInputPayload, keyboard, 0);
CHECK_OFFSET(KVMFRInputPayload, reserved, 0);
CHECK_ARRAY(KVMFRInputPayload, reserved, 16);

CHECK_SIZE(KVMFRInputMessage, 32);
CHECK_ALIGN(KVMFRInputMessage, 4);
CHECK_OFFSET(KVMFRInputMessage, type, 0);
CHECK_OFFSET(KVMFRInputMessage, generation, 4);
CHECK_OFFSET(KVMFRInputMessage, sequence, 8);
CHECK_OFFSET(KVMFRInputMessage, reserved, 12);
CHECK_OFFSET(KVMFRInputMessage, payload, 16);

CHECK_SIZE(KVMFRInputStreamEndpoint, 48);
CHECK_ALIGN(KVMFRInputStreamEndpoint, 4);
CHECK_OFFSET(KVMFRInputStreamEndpoint, stream, 0);
CHECK_OFFSET(KVMFRInputStreamEndpoint, boundClientID, 32);
CHECK_OFFSET(KVMFRInputStreamEndpoint, bindingGeneration, 36);
CHECK_OFFSET(KVMFRInputStreamEndpoint, flags, 40);
CHECK_OFFSET(KVMFRInputStreamEndpoint, reserved, 44);

CHECK_SIZE(KVMFRInputStatus, 448);
CHECK_ALIGN(KVMFRInputStatus, 4);
CHECK_OFFSET(KVMFRInputStatus, version, 0);
CHECK_OFFSET(KVMFRInputStatus, capabilities, 4);
CHECK_OFFSET(KVMFRInputStatus, flags, 8);
CHECK_OFFSET(KVMFRInputStatus, generation, 12);
CHECK_OFFSET(KVMFRInputStatus, ownerClientID, 16);
CHECK_OFFSET(KVMFRInputStatus, ownerGeneration, 20);
CHECK_OFFSET(KVMFRInputStatus, lease, 24);
CHECK_OFFSET(KVMFRInputStatus, maxButtons, 28);
CHECK_OFFSET(KVMFRInputStatus, streamVersion, 32);
CHECK_OFFSET(KVMFRInputStatus, streamEndpointCount, 36);
CHECK_OFFSET(KVMFRInputStatus, streamGeneration, 40);
CHECK_OFFSET(KVMFRInputStatus, keyboardLEDs, 44);
CHECK_OFFSET(KVMFRInputStatus, statusReserved, 45);
CHECK_OFFSET(KVMFRInputStatus, streamReserved, 48);
CHECK_OFFSET(KVMFRInputStatus, streamEndpoint, 64);
CHECK_ARRAY(KVMFRInputStatus, statusReserved, 3);
CHECK_ARRAY(KVMFRInputStatus, streamReserved, 4);
CHECK_ARRAY(KVMFRInputStatus, streamEndpoint, 8);

CHECK_CONST(KVMFR_CLIPBOARD_VERSION, 4);
CHECK_CONST(KVMFR_CLIPBOARD_STREAM_VERSION, 1);
CHECK_CONST(KVMFR_CLIPBOARD_STREAM_SLOT_COUNT, 4);
CHECK_CONST(KVMFR_CLIPBOARD_STREAM_SLOT_BYTES, 262144u);
CHECK_CONST(KVMFR_CLIPBOARD_STREAM_WINDOW_BYTES, 1048576u);
CHECK_CONST(KVMFR_CLIPBOARD_SLOT_COUNT, 4);
CHECK_CONST(KVMFR_CLIPBOARD_DATA_BYTES, 262144u);
CHECK_CONST(KVMFR_CLIPBOARD_REPRESENTATION_BYTES, 65536u);
CHECK_CONST(KVMFR_CLIPBOARD_SIZE_UNKNOWN, UINT64_MAX);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_READ_BYTES, 1048576u);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ROOT_NODE, UINT64_C(0));
CHECK_CONST(KVMFR_CLIPBOARD_FILE_MAX_ACQUISITIONS, 8);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_MAX_REQUESTS, 32);
CHECK_CONST(KVMFR_CLIPBOARD_TRANSFER_HELPER, UINT64_C(9223372036854775808));
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ENTRY_ALIGN, 8u);
CHECK_CONST(KVMFR_CLIPBOARD_STREAM_RECORD_BYTES, 262208u);

CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_NONE, 0);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_TEXT, 1);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_PNG, 2);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_BMP, 3);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_TIFF, 4);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_JPEG, 5);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_FILES, 6);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_TEXT, 1);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_PNG, 2);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_BMP, 4);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_TIFF, 8);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_JPEG, 16);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_FILES, 32);
CHECK_CONST(KVMFR_CLIPBOARD_FORMAT_MASK_ALL, 63);

CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_CLAIM, 1);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_RELEASE, 2);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_KEEPALIVE, 3);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_OFFER, 4);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_CLEAR, 5);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_REQUEST, 6);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_DATA, 7);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_CANCEL, 8);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE, 9);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRED, 10);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_FILE_RELEASE, 11);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_FILE_REQUEST, 12);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_FILE_DATA, 13);
CHECK_CONST(KVMFR_CLIPBOARD_MESSAGE_FILE_CANCEL, 14);
CHECK_CONST(KVMFR_CLIPBOARD_FLAG_BEGIN, 1);
CHECK_CONST(KVMFR_CLIPBOARD_FLAG_END, 2);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_OP_LIST, 1);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_OP_READ, 2);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_TYPE_REGULAR, 1);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_TYPE_DIRECTORY, 2);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_NONE, 0);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_NOT_FOUND, 1);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_ACCESS, 2);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_NOT_DIRECTORY, 3);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_IS_DIRECTORY, 4);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_IO, 5);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_INVALID, 6);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_NO_MEMORY, 7);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_NO_SPACE, 8);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_DISCONNECTED, 9);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_CANCELLED, 10);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_NOT_SUPPORTED, 11);
CHECK_CONST(KVMFR_CLIPBOARD_FILE_ERROR_STALE, 12);
CHECK_CONST(KVMFR_CLIPBOARD_STATUS_AVAILABLE, 1);
CHECK_CONST(KVMFR_CLIPBOARD_STATUS_HAS_OWNER, 2);
CHECK_CONST(KVMFR_CLIPBOARD_QUEUE_STATUS, 1);
CHECK_CONST(KVMFR_CLIPBOARD_QUEUE_MESSAGE, 2);

CHECK_SIZE(KVMFRClipboardFormat, 4);
CHECK_SIZE(KVMFRClipboardFormatFlags, 4);
CHECK_SIZE(KVMFRClipboardMessageType, 4);
CHECK_SIZE(KVMFRClipboardFlags, 4);
CHECK_SIZE(KVMFRClipboardFileOperation, 4);
CHECK_SIZE(KVMFRClipboardFileType, 4);
CHECK_SIZE(KVMFRClipboardFileError, 4);
CHECK_SIZE(KVMFRClipboardStatusFlags, 4);
CHECK_SIZE(KVMFRClipboardQueueType, 4);

CHECK_SIZE(KVMFRClipboardMessage, 64);
CHECK_ALIGN(KVMFRClipboardMessage, 8);
CHECK_OFFSET(KVMFRClipboardMessage, version, 0);
CHECK_OFFSET(KVMFRClipboardMessage, type, 4);
CHECK_OFFSET(KVMFRClipboardMessage, generation, 8);
CHECK_OFFSET(KVMFRClipboardMessage, sequence, 12);
CHECK_OFFSET(KVMFRClipboardMessage, clipboardGeneration, 16);
CHECK_OFFSET(KVMFRClipboardMessage, transfer, 24);
CHECK_OFFSET(KVMFRClipboardMessage, offset, 32);
CHECK_OFFSET(KVMFRClipboardMessage, size, 40);
CHECK_OFFSET(KVMFRClipboardMessage, format, 48);
CHECK_OFFSET(KVMFRClipboardMessage, flags, 52);
CHECK_OFFSET(KVMFRClipboardMessage, token, 56);
CHECK_OFFSET(KVMFRClipboardMessage, length, 60);

CHECK_SIZE(KVMFRClipboardFileEntry, 40);
CHECK_ALIGN(KVMFRClipboardFileEntry, 8);
CHECK_OFFSET(KVMFRClipboardFileEntry, node, 0);
CHECK_OFFSET(KVMFRClipboardFileEntry, size, 8);
CHECK_OFFSET(KVMFRClipboardFileEntry, createdNs, 16);
CHECK_OFFSET(KVMFRClipboardFileEntry, modifiedNs, 24);
CHECK_OFFSET(KVMFRClipboardFileEntry, type, 32);
CHECK_OFFSET(KVMFRClipboardFileEntry, nameLength, 36);

CHECK_SIZE(KVMFRClipboardStatus, 128);
CHECK_ALIGN(KVMFRClipboardStatus, 4);
CHECK_OFFSET(KVMFRClipboardStatus, version, 0);
CHECK_OFFSET(KVMFRClipboardStatus, flags, 4);
CHECK_OFFSET(KVMFRClipboardStatus, generation, 8);
CHECK_OFFSET(KVMFRClipboardStatus, ownerClientID, 12);
CHECK_OFFSET(KVMFRClipboardStatus, ownerGeneration, 16);
CHECK_OFFSET(KVMFRClipboardStatus, lease, 20);
CHECK_OFFSET(KVMFRClipboardStatus, formats, 24);
CHECK_OFFSET(KVMFRClipboardStatus, slotBytes, 28);
CHECK_OFFSET(KVMFRClipboardStatus, streamVersion, 32);
CHECK_OFFSET(KVMFRClipboardStatus, streamSlotCount, 36);
CHECK_OFFSET(KVMFRClipboardStatus, reserved, 40);
CHECK_OFFSET(KVMFRClipboardStatus, hostToClient, 64);
CHECK_OFFSET(KVMFRClipboardStatus, clientToHost, 96);
CHECK_ARRAY(KVMFRClipboardStatus, reserved, 6);

CHECK_SIZE(KVMFRClipboardSlotHeader, 64);
CHECK_ALIGN(KVMFRClipboardSlotHeader, 8);
#if defined(__cplusplus)
ABI_ASSERT((std::is_same<KVMFRClipboardSlotHeader,
  KVMFRClipboardMessage>::value));
#else
ABI_ASSERT(_Generic((KVMFRClipboardSlotHeader *)0,
  KVMFRClipboardMessage *: 1, default: 0));
#endif

ABI_ASSERT(KVMFR_CLIPBOARD_FILE_ENTRY_BYTES(0) == 40);
ABI_ASSERT(KVMFR_CLIPBOARD_FILE_ENTRY_BYTES(1) == 48);
ABI_ASSERT(KVMFR_CLIPBOARD_FILE_ENTRY_BYTES(7) == 48);
ABI_ASSERT(KVMFR_CLIPBOARD_FILE_ENTRY_BYTES(8) == 48);
ABI_ASSERT(KVMFR_CLIPBOARD_FILE_ENTRY_BYTES(9) == 56);
ABI_ASSERT(KVMFR_CLIPBOARD_QUEUE_UDATA(1, 2) == UINT64_C(0x0000000100000002));
ABI_ASSERT(KVMFR_CLIPBOARD_QUEUE_TYPE(UINT64_C(0x123456789abcdef0)) ==
  UINT32_C(0x12345678));
ABI_ASSERT(KVMFR_CLIPBOARD_QUEUE_SERIAL(UINT64_C(0x123456789abcdef0)) ==
  UINT32_C(0x9abcdef0));

CHECK_CONST(KVMFR_R_VERSION, 1);
CHECK_CONST(KVMFR_R_READY, 1);
CHECK_CONST(KVMFR_R_REGION_SIZE, 65536u);
CHECK_CONST(KVMFR_R_LINE_SIZE, 64u);
CHECK_CONST(KVMFR_R_HEARTBEAT_MS, 250u);
CHECK_CONST(KVMFR_R_REQ_SLOTS, 16);
CHECK_CONST(KVMFR_R_CAP_DISPLAY, 1);
CHECK_CONST(KVMFR_R_REQ_WRITING, 1);
CHECK_CONST(KVMFR_R_REQ_FIRST, 2);
CHECK_CONST(KVMFR_R_REQ_NONE, 0);
CHECK_CONST(KVMFR_R_REQ_NORMAL, 1);
CHECK_CONST(KVMFR_R_REQ_RECOVERY, 2);
CHECK_CONST(KVMFR_R_STATE_UNKNOWN, 0);
CHECK_CONST(KVMFR_R_STATE_NORMAL, 1);
CHECK_CONST(KVMFR_R_STATE_SWITCHING, 2);
CHECK_CONST(KVMFR_R_STATE_ACTIVE, 3);
CHECK_CONST(KVMFR_R_STATE_FAILED, 4);
CHECK_CONST(KVMFR_R_ERR_NONE, 0);
CHECK_CONST(KVMFR_R_ERR_UNSUPPORTED, 1);
CHECK_CONST(KVMFR_R_ERR_HELPER_UNAVAILABLE, 2);
CHECK_CONST(KVMFR_R_ERR_TOPOLOGY_FAILED, 3);
CHECK_CONST(KVMFR_R_ERR_NO_FALLBACK_DISPLAY, 4);
CHECK_CONST(KVMFR_R_ERR_BUSY, 5);
CHECK_CONST(KVMFR_R_ERR_CAPACITY, 6);
ABI_ASSERT(sizeof(KVMFR_R_MAGIC) == 9);

CHECK_SIZE(KVMFRRHeader, 64);
CHECK_ALIGN(KVMFRRHeader, 8);
CHECK_OFFSET(KVMFRRHeader, magic, 0);
CHECK_OFFSET(KVMFRRHeader, abiVersion, 8);
CHECK_OFFSET(KVMFRRHeader, structSize, 10);
CHECK_OFFSET(KVMFRRHeader, capabilities, 12);
CHECK_OFFSET(KVMFRRHeader, lgmpVersion, 16);
CHECK_OFFSET(KVMFRRHeader, kvmfrVersion, 20);
CHECK_OFFSET(KVMFRRHeader, session, 24);
CHECK_OFFSET(KVMFRRHeader, uuid, 32);
CHECK_OFFSET(KVMFRRHeader, heartbeat, 48);
CHECK_OFFSET(KVMFRRHeader, reserved, 52);
CHECK_OFFSET(KVMFRRHeader, ready, 60);
CHECK_ARRAY(KVMFRRHeader, magic, 8);
CHECK_ARRAY(KVMFRRHeader, uuid, 16);
CHECK_ARRAY(KVMFRRHeader, reserved, 2);

CHECK_SIZE(KVMFRRInfo, 64);
CHECK_ALIGN(KVMFRRInfo, 1);
CHECK_OFFSET(KVMFRRInfo, version, 0);
CHECK_OFFSET(KVMFRRInfo, reserved, 48);
CHECK_ARRAY(KVMFRRInfo, version, 48);
CHECK_ARRAY(KVMFRRInfo, reserved, 16);

CHECK_SIZE(KVMFRRReqHead, 64);
CHECK_ALIGN(KVMFRRReqHead, 4);
CHECK_OFFSET(KVMFRRReqHead, ticket, 0);
CHECK_OFFSET(KVMFRRReqHead, reserved, 4);
CHECK_ARRAY(KVMFRRReqHead, reserved, 60);

CHECK_SIZE(KVMFRRRequest, 64);
CHECK_ALIGN(KVMFRRRequest, 8);
CHECK_OFFSET(KVMFRRRequest, serial, 0);
CHECK_OFFSET(KVMFRRRequest, request, 4);
CHECK_OFFSET(KVMFRRRequest, session, 8);
CHECK_OFFSET(KVMFRRRequest, reserved, 16);
CHECK_ARRAY(KVMFRRRequest, reserved, 48);

CHECK_SIZE(KVMFRRStatus, 64);
CHECK_ALIGN(KVMFRRStatus, 8);
CHECK_OFFSET(KVMFRRStatus, ackSerial, 0);
CHECK_OFFSET(KVMFRRStatus, ackRequest, 4);
CHECK_OFFSET(KVMFRRStatus, state, 8);
CHECK_OFFSET(KVMFRRStatus, error, 12);
CHECK_OFFSET(KVMFRRStatus, session, 16);
CHECK_OFFSET(KVMFRRStatus, serial, 24);
CHECK_OFFSET(KVMFRRStatus, reserved, 28);
CHECK_ARRAY(KVMFRRStatus, reserved, 36);

CHECK_SIZE(KVMFRR, 1280);
CHECK_ALIGN(KVMFRR, 8);
CHECK_OFFSET(KVMFRR, header, 0);
CHECK_OFFSET(KVMFRR, info, 64);
CHECK_OFFSET(KVMFRR, req, 128);
CHECK_OFFSET(KVMFRR, requests, 192);
CHECK_OFFSET(KVMFRR, status, 1216);
CHECK_ARRAY(KVMFRR, requests, 16);

CHECK_CONST(KVMFR_VERSION, 34);
CHECK_CONST(KVMFR_SDR_WHITE_LEVEL_DEFAULT, 203);
CHECK_CONST(KVMFR_MAX_DAMAGE_RECTS, 64);
CHECK_CONST(CURSOR_FLAG_POSITION, 1);
CHECK_CONST(CURSOR_FLAG_VISIBLE, 2);
CHECK_CONST(CURSOR_FLAG_SHAPE, 4);
CHECK_CONST(CURSOR_FLAG_COLOR_TRANSFORM, 8);
CHECK_CONST(CURSOR_FLAG_VISIBLE_VALID, 16);
CHECK_CONST(KVMFR_FEATURE_SETCURSORPOS, 1);
CHECK_CONST(KVMFR_FEATURE_WINDOWSIZE, 2);
CHECK_CONST(KVMFR_FEATURE_FRAME_SCHEDULE, 4);
CHECK_CONST(KVMFR_FEATURE_INPUT, 8);
CHECK_CONST(KVMFR_FEATURE_CLIPBOARD, 16);
CHECK_CONST(KVMFR_MESSAGE_SETCURSORPOS, 0);
CHECK_CONST(KVMFR_MESSAGE_WINDOWSIZE, 1);
CHECK_CONST(KVMFR_MESSAGE_FRAME_SCHEDULE, 2);
CHECK_CONST(KVMFR_RECORD_VMINFO, 1);
CHECK_CONST(KVMFR_RECORD_OSINFO, 2);
CHECK_CONST(KVMFR_OS_LINUX, 0);
CHECK_CONST(KVMFR_OS_BSD, 1);
CHECK_CONST(KVMFR_OS_OSX, 2);
CHECK_CONST(KVMFR_OS_WINDOWS, 3);
CHECK_CONST(KVMFR_OS_OTHER, 4);
CHECK_CONST(KVMFR_COLOR_TRANSFORM_MATRIX, 1);
CHECK_CONST(KVMFR_COLOR_TRANSFORM_LUT, 2);
CHECK_CONST(FRAME_FLAG_BLOCK_SCREENSAVER, 1);
CHECK_CONST(FRAME_FLAG_REQUEST_ACTIVATION, 2);
CHECK_CONST(FRAME_FLAG_TRUNCATED, 4);
CHECK_CONST(FRAME_FLAG_HDR, 8);
CHECK_CONST(FRAME_FLAG_HDR_PQ, 16);
CHECK_CONST(FRAME_FLAG_HDR_METADATA, 32);
CHECK_CONST(KVMFR_FRAME_TIMING_PHASE_VALID, 1);
CHECK_CONST(KVMFR_FRAME_SCHEDULE_ACTIVE, 1);
CHECK_CONST(KVMFR_FRAME_SCHEDULE_RELEASE, 2);
CHECK_CONST(KVMFR_FRAME_SCHEDULE_RESET, 4);
CHECK_CONST(KVMFR_FRAME_SCHEDULE_IMMEDIATE, 8);
CHECK_CONST(KVMFR_FRAMEBUFFER_WP_SIZE, 4u);
ABI_ASSERT(sizeof(KVMFR_MAGIC) == 9);

CHECK_SIZE(KVMFRCursorFlags, 4);
CHECK_SIZE(KVMFRFeatureFlags, 4);
CHECK_SIZE(KVMFRMessageType, 4);
CHECK_SIZE(KVMFRColorTransformFlags, 4);
CHECK_SIZE(KVMFRFrameFlags, 4);
CHECK_SIZE(KVMFRFrameTimingFlags, 4);
CHECK_SIZE(KVMFRFrameScheduleFlags, 4);
CHECK_SIZE(KVMFROS, 4);

CHECK_SIZE(KVMFR, 48);
CHECK_ALIGN(KVMFR, 4);
CHECK_OFFSET(KVMFR, magic, 0);
CHECK_OFFSET(KVMFR, version, 8);
CHECK_OFFSET(KVMFR, hostver, 12);
CHECK_OFFSET(KVMFR, features, 44);
CHECK_ARRAY(KVMFR, magic, 8);
CHECK_ARRAY(KVMFR, hostver, 32);

CHECK_SIZE(KVMFRRecord, 8);
CHECK_ALIGN(KVMFRRecord, 4);
CHECK_OFFSET(KVMFRRecord, type, 0);
CHECK_OFFSET(KVMFRRecord, size, 4);
CHECK_OFFSET(KVMFRRecord, data, 8);

CHECK_SIZE(KVMFRRecord_VMInfo, 51);
CHECK_ALIGN(KVMFRRecord_VMInfo, 1);
CHECK_OFFSET(KVMFRRecord_VMInfo, uuid, 0);
CHECK_OFFSET(KVMFRRecord_VMInfo, capture, 16);
CHECK_OFFSET(KVMFRRecord_VMInfo, cpus, 48);
CHECK_OFFSET(KVMFRRecord_VMInfo, cores, 49);
CHECK_OFFSET(KVMFRRecord_VMInfo, sockets, 50);
CHECK_OFFSET(KVMFRRecord_VMInfo, model, 51);
CHECK_ARRAY(KVMFRRecord_VMInfo, uuid, 16);
CHECK_ARRAY(KVMFRRecord_VMInfo, capture, 32);

CHECK_SIZE(KVMFRRecord_OSInfo, 1);
CHECK_ALIGN(KVMFRRecord_OSInfo, 1);
CHECK_OFFSET(KVMFRRecord_OSInfo, os, 0);
CHECK_OFFSET(KVMFRRecord_OSInfo, name, 1);

CHECK_SIZE(KVMFRCursor, 28);
CHECK_ALIGN(KVMFRCursor, 4);
CHECK_OFFSET(KVMFRCursor, x, 0);
CHECK_OFFSET(KVMFRCursor, y, 2);
CHECK_OFFSET(KVMFRCursor, type, 4);
CHECK_OFFSET(KVMFRCursor, hx, 8);
CHECK_OFFSET(KVMFRCursor, hy, 9);
CHECK_OFFSET(KVMFRCursor, width, 12);
CHECK_OFFSET(KVMFRCursor, height, 16);
CHECK_OFFSET(KVMFRCursor, pitch, 20);
CHECK_OFFSET(KVMFRCursor, sdrWhiteLevel, 24);

CHECK_SIZE(KVMFRFrameBufferWritePointer, 4);
CHECK_ALIGN(KVMFRFrameBufferWritePointer, 4);
#if defined(__GNUC__) || defined(__clang__)
ABI_ASSERT(__atomic_always_lock_free(
  sizeof(KVMFRFrameBufferWritePointer), 0));
#endif
CHECK_SIZE(KVMFRFrameBuffer, 4);
CHECK_ALIGN(KVMFRFrameBuffer, 4);
CHECK_OFFSET(KVMFRFrameBuffer, wp, 0);
CHECK_OFFSET(KVMFRFrameBuffer, data, 4);
ABI_ASSERT(sizeof(((KVMFRFrameBuffer *)0)->data) == 0);

CHECK_SIZE(KVMFRFrame, 1216);
CHECK_ALIGN(KVMFRFrame, 8);
CHECK_OFFSET(KVMFRFrame, formatVer, 0);
CHECK_OFFSET(KVMFRFrame, frameSerial, 4);
CHECK_OFFSET(KVMFRFrame, type, 8);
CHECK_OFFSET(KVMFRFrame, screenWidth, 12);
CHECK_OFFSET(KVMFRFrame, screenHeight, 16);
CHECK_OFFSET(KVMFRFrame, dataWidth, 20);
CHECK_OFFSET(KVMFRFrame, dataHeight, 24);
CHECK_OFFSET(KVMFRFrame, frameWidth, 28);
CHECK_OFFSET(KVMFRFrame, frameHeight, 32);
CHECK_OFFSET(KVMFRFrame, rotation, 36);
CHECK_OFFSET(KVMFRFrame, stride, 40);
CHECK_OFFSET(KVMFRFrame, pitch, 44);
CHECK_OFFSET(KVMFRFrame, offset, 48);
CHECK_OFFSET(KVMFRFrame, flags, 52);
CHECK_OFFSET(KVMFRFrame, damageRectsCount, 56);
CHECK_OFFSET(KVMFRFrame, sdrWhiteLevel, 60);
CHECK_OFFSET(KVMFRFrame, captureTime, 64);
CHECK_OFFSET(KVMFRFrame, postProcessTime, 72);
CHECK_OFFSET(KVMFRFrame, copyTime, 80);
CHECK_OFFSET(KVMFRFrame, readyTime, 88);
CHECK_OFFSET(KVMFRFrame, holdTime, 96);
CHECK_OFFSET(KVMFRFrame, readyLeadTime, 104);
CHECK_OFFSET(KVMFRFrame, timingSerial, 112);
CHECK_OFFSET(KVMFRFrame, timingValid, 116);
CHECK_OFFSET(KVMFRFrame, timingFlags, 120);
CHECK_OFFSET(KVMFRFrame, timingReserved, 124);
CHECK_OFFSET(KVMFRFrame, hdrDisplayPrimary, 128);
CHECK_OFFSET(KVMFRFrame, hdrWhitePoint, 140);
CHECK_OFFSET(KVMFRFrame, hdrMaxDisplayLuminance, 144);
CHECK_OFFSET(KVMFRFrame, hdrMinDisplayLuminance, 148);
CHECK_OFFSET(KVMFRFrame, hdrMaxContentLightLevel, 152);
CHECK_OFFSET(KVMFRFrame, hdrMaxFrameAverageLightLevel, 156);
CHECK_OFFSET(KVMFRFrame, scheduleGeneration, 160);
CHECK_OFFSET(KVMFRFrame, scheduleEpoch, 164);
CHECK_OFFSET(KVMFRFrame, scheduleDeadlineSerial, 168);
CHECK_OFFSET(KVMFRFrame, hdrReserved, 172);
CHECK_OFFSET(KVMFRFrame, damageRects, 192);
CHECK_ARRAY(KVMFRFrame, timingReserved, 4);
CHECK_ARRAY(KVMFRFrame, hdrDisplayPrimary, 3);
ABI_ASSERT(sizeof(((KVMFRFrame *)0)->hdrDisplayPrimary[0]) /
  sizeof(uint16_t) == 2);
CHECK_ARRAY(KVMFRFrame, hdrWhitePoint, 2);
CHECK_ARRAY(KVMFRFrame, hdrReserved, 20);
CHECK_ARRAY(KVMFRFrame, damageRects, 64);

CHECK_SIZE(KVMFRMessage, 4);
CHECK_ALIGN(KVMFRMessage, 4);
CHECK_OFFSET(KVMFRMessage, type, 0);

CHECK_SIZE(KVMFRSetCursorPos, 12);
CHECK_ALIGN(KVMFRSetCursorPos, 4);
CHECK_OFFSET(KVMFRSetCursorPos, msg, 0);
CHECK_OFFSET(KVMFRSetCursorPos, x, 4);
CHECK_OFFSET(KVMFRSetCursorPos, y, 8);

CHECK_SIZE(KVMFRWindowSize, 12);
CHECK_ALIGN(KVMFRWindowSize, 4);
CHECK_OFFSET(KVMFRWindowSize, msg, 0);
CHECK_OFFSET(KVMFRWindowSize, w, 4);
CHECK_OFFSET(KVMFRWindowSize, h, 8);

CHECK_SIZE(KVMFRFrameSchedule, 64);
CHECK_ALIGN(KVMFRFrameSchedule, 8);
CHECK_OFFSET(KVMFRFrameSchedule, msg, 0);
CHECK_OFFSET(KVMFRFrameSchedule, clientID, 4);
CHECK_OFFSET(KVMFRFrameSchedule, generation, 8);
CHECK_OFFSET(KVMFRFrameSchedule, flags, 12);
CHECK_OFFSET(KVMFRFrameSchedule, period, 16);
CHECK_OFFSET(KVMFRFrameSchedule, targetSlack, 24);
CHECK_OFFSET(KVMFRFrameSchedule, phaseError, 32);
CHECK_OFFSET(KVMFRFrameSchedule, feedbackFrameSerial, 40);
CHECK_OFFSET(KVMFRFrameSchedule, feedbackScheduleEpoch, 44);
CHECK_OFFSET(KVMFRFrameSchedule, feedbackDeadlineSerial, 48);
CHECK_OFFSET(KVMFRFrameSchedule, lease, 52);
CHECK_OFFSET(KVMFRFrameSchedule, reserved, 56);
CHECK_ARRAY(KVMFRFrameSchedule, reserved, 8);

#define RUNTIME_CHECK(condition) \
  do                             \
  {                              \
    if (!(condition))            \
      return __LINE__;           \
  }                              \
  while (0)

static void initializeFileMessage(KVMFRClipboardMessage * message,
  KVMFRClipboardMessageType type)
{
  memset(message, 0, sizeof(*message));
  message->version             = KVMFR_CLIPBOARD_VERSION;
  message->type                = type;
  message->clipboardGeneration = 1;
  message->transfer            = 1;
  message->format              = KVMFR_CLIPBOARD_FORMAT_FILES;
}

static int checkClipboardHelpers(void)
{
  RUNTIME_CHECK(!kvmfrClipboardTransferFromHelper(0));
  RUNTIME_CHECK(!kvmfrClipboardTransferFromHelper(1));
  RUNTIME_CHECK(kvmfrClipboardTransferFromHelper(
    KVMFR_CLIPBOARD_TRANSFER_HELPER));
  RUNTIME_CHECK(kvmfrClipboardTransferFromHelper(
    KVMFR_CLIPBOARD_TRANSFER_HELPER | UINT64_C(17)));

  RUNTIME_CHECK(!kvmfrClipboardTransferFromClient(0));
  RUNTIME_CHECK(kvmfrClipboardTransferFromClient(1));
  RUNTIME_CHECK(!kvmfrClipboardTransferFromClient(
    KVMFR_CLIPBOARD_TRANSFER_HELPER));

  RUNTIME_CHECK(!kvmfrClipboardFormatValid(KVMFR_CLIPBOARD_FORMAT_NONE));
  RUNTIME_CHECK(kvmfrClipboardFormatValid(KVMFR_CLIPBOARD_FORMAT_TEXT));
  RUNTIME_CHECK(kvmfrClipboardFormatValid(KVMFR_CLIPBOARD_FORMAT_FILES));
  RUNTIME_CHECK(!kvmfrClipboardFormatValid(7));

  RUNTIME_CHECK(!kvmfrClipboardRepresentationFormatValid(
    KVMFR_CLIPBOARD_FORMAT_NONE));
  RUNTIME_CHECK(kvmfrClipboardRepresentationFormatValid(
    KVMFR_CLIPBOARD_FORMAT_TEXT));
  RUNTIME_CHECK(kvmfrClipboardRepresentationFormatValid(
    KVMFR_CLIPBOARD_FORMAT_JPEG));
  RUNTIME_CHECK(!kvmfrClipboardRepresentationFormatValid(
    KVMFR_CLIPBOARD_FORMAT_FILES));

  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_NONE) == 0);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_TEXT) == 1);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_PNG) == 2);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_BMP) == 4);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_TIFF) == 8);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_JPEG) == 16);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(KVMFR_CLIPBOARD_FORMAT_FILES) == 32);
  RUNTIME_CHECK(kvmfrClipboardFormatFlag(7) == 0);

  RUNTIME_CHECK(kvmfrClipboardFileErrorValid(
    KVMFR_CLIPBOARD_FILE_ERROR_NONE));
  RUNTIME_CHECK(kvmfrClipboardFileErrorValid(
    KVMFR_CLIPBOARD_FILE_ERROR_STALE));
  RUNTIME_CHECK(!kvmfrClipboardFileErrorValid(13));
  RUNTIME_CHECK(!kvmfrClipboardFileOperationValid(0));
  RUNTIME_CHECK(kvmfrClipboardFileOperationValid(
    KVMFR_CLIPBOARD_FILE_OP_LIST));
  RUNTIME_CHECK(kvmfrClipboardFileOperationValid(
    KVMFR_CLIPBOARD_FILE_OP_READ));
  RUNTIME_CHECK(!kvmfrClipboardFileOperationValid(3));
  RUNTIME_CHECK(!kvmfrClipboardFileTransferValid(0));
  RUNTIME_CHECK(kvmfrClipboardFileTransferValid(1));
  RUNTIME_CHECK(kvmfrClipboardFileTransferValid(
    KVMFR_CLIPBOARD_TRANSFER_HELPER));
  RUNTIME_CHECK(!kvmfrClipboardFileMessageType(
    KVMFR_CLIPBOARD_MESSAGE_CANCEL));
  RUNTIME_CHECK(kvmfrClipboardFileMessageType(
    KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE));
  RUNTIME_CHECK(kvmfrClipboardFileMessageType(
    KVMFR_CLIPBOARD_MESSAGE_FILE_CANCEL));
  RUNTIME_CHECK(!kvmfrClipboardFileMessageType(15));

  return 0;
}

static int checkClipboardCommonHeader(void)
{
  KVMFRClipboardMessage message;

  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(NULL));

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE);
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  message.version = KVMFR_CLIPBOARD_VERSION - 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.version = KVMFR_CLIPBOARD_VERSION;

  message.type = KVMFR_CLIPBOARD_MESSAGE_CLAIM;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.type = KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE;

  message.clipboardGeneration = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.clipboardGeneration = 1;

  message.transfer = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.transfer = 1;

  message.format = KVMFR_CLIPBOARD_FORMAT_TEXT;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.format = KVMFR_CLIPBOARD_FORMAT_FILES;

  message.length = KVMFR_CLIPBOARD_DATA_BYTES + 1u;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  return 0;
}

static int checkClipboardAcquireMessages(void)
{
  KVMFRClipboardMessage message;

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE);
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  message.sequence = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.sequence = 0;
  message.offset   = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.offset = 0;
  message.size   = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.size  = 0;
  message.flags = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.flags = 0;
  message.token = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.token  = 0;
  message.length = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_RELEASE);
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRED);
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.token = KVMFR_CLIPBOARD_FILE_ERROR_STALE;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.token = KVMFR_CLIPBOARD_FILE_ERROR_STALE + 1u;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.token    = KVMFR_CLIPBOARD_FILE_ERROR_NONE;
  message.sequence = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  return 0;
}

static int checkClipboardRequestMessages(void)
{
  KVMFRClipboardMessage message;

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_REQUEST);
  message.token = KVMFR_CLIPBOARD_FILE_OP_LIST;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.size = UINT64_MAX;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  message.offset = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.offset = 0;
  message.flags  = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.flags    = 0;
  message.sequence = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.sequence = 0;
  message.length   = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.length = 0;
  message.token  = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.token  = KVMFR_CLIPBOARD_FILE_OP_READ;
  message.size   = 1;
  message.flags  = 1;
  message.offset = UINT64_MAX;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  message.flags  = KVMFR_CLIPBOARD_FILE_READ_BYTES;
  message.offset = UINT64_MAX - (KVMFR_CLIPBOARD_FILE_READ_BYTES - 1u);
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  ++message.offset;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.offset = 0;
  message.flags  = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.flags = KVMFR_CLIPBOARD_FILE_READ_BYTES + 1u;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.flags = 1;
  message.size  = KVMFR_CLIPBOARD_FILE_ROOT_NODE;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  return 0;
}

static int checkClipboardDataMessages(void)
{
  KVMFRClipboardMessage message;

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_DATA);
  message.token    = KVMFR_CLIPBOARD_FILE_OP_LIST;
  message.flags    = KVMFR_CLIPBOARD_FLAG_BEGIN;
  message.offset   = 5;
  message.size     = 8;
  message.length   = 3;
  message.sequence = UINT32_MAX;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  message.size = KVMFR_CLIPBOARD_SIZE_UNKNOWN;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.size = 7;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.flags = 0;
  message.size  = KVMFR_CLIPBOARD_SIZE_UNKNOWN;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.size = 8;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.flags  = KVMFR_CLIPBOARD_FLAG_END;
  message.size   = 8;
  message.length = 3;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.size = 9;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.size   = 5;
  message.length = 0;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.flags = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.flags  = KVMFR_CLIPBOARD_FLAG_BEGIN | KVMFR_CLIPBOARD_FLAG_END;
  message.offset = 5;
  message.length = 3;
  message.size   = 8;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.size = KVMFR_CLIPBOARD_SIZE_UNKNOWN;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.flags = 4;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.flags = KVMFR_CLIPBOARD_FLAG_END;
  message.token = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.token  = KVMFR_CLIPBOARD_FILE_OP_READ;
  message.offset = UINT64_MAX;
  message.length = 1;
  message.size   = 0;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.length = 0;
  message.size   = UINT64_MAX;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));

  return 0;
}

static int checkClipboardCancelMessages(void)
{
  KVMFRClipboardMessage message;

  initializeFileMessage(&message, KVMFR_CLIPBOARD_MESSAGE_FILE_CANCEL);
  message.token = KVMFR_CLIPBOARD_FILE_ERROR_NOT_FOUND;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.token = KVMFR_CLIPBOARD_FILE_ERROR_STALE;
  RUNTIME_CHECK(kvmfrClipboardFileMessageValid(&message));
  message.token = KVMFR_CLIPBOARD_FILE_ERROR_NONE;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.token = KVMFR_CLIPBOARD_FILE_ERROR_STALE + 1u;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  message.token    = KVMFR_CLIPBOARD_FILE_ERROR_IO;
  message.sequence = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.sequence = 0;
  message.offset   = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.offset = 0;
  message.size   = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.size  = 0;
  message.flags = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));
  message.flags  = 0;
  message.length = 1;
  RUNTIME_CHECK(!kvmfrClipboardFileMessageValid(&message));

  return 0;
}

static int checkPlatformProperties(void)
{
  const uint16_t endianProbe = 1;

  RUNTIME_CHECK(*(const uint8_t *)&endianProbe == 1);
  RUNTIME_CHECK(memcmp(KVMFR_MAGIC, "KVMFR---", 8) == 0);
  RUNTIME_CHECK(KVMFR_MAGIC[8] == '\0');
  RUNTIME_CHECK(memcmp(KVMFR_R_MAGIC, "KVMFRRCV", 8) == 0);
  RUNTIME_CHECK(KVMFR_R_MAGIC[8] == '\0');

#if defined(__cplusplus)
  KVMFRFrameBufferWritePointer pointer(0);

  RUNTIME_CHECK(pointer.is_lock_free());
  pointer.store(UINT32_MAX, std::memory_order_release);
  RUNTIME_CHECK(pointer.load(std::memory_order_acquire) == UINT32_MAX);
#else
  KVMFRFrameBufferWritePointer pointer;

  atomic_init(&pointer, 0);
  RUNTIME_CHECK(atomic_is_lock_free(&pointer));
  atomic_store_explicit(&pointer, UINT32_MAX, memory_order_release);
  RUNTIME_CHECK(atomic_load_explicit(&pointer, memory_order_acquire) ==
    UINT32_MAX);
#endif

  return 0;
}

int main(void)
{
  int result;

  result = checkClipboardHelpers();
  if (result != 0)
    return result;

  result = checkClipboardCommonHeader();
  if (result != 0)
    return result;

  result = checkClipboardAcquireMessages();
  if (result != 0)
    return result;

  result = checkClipboardRequestMessages();
  if (result != 0)
    return result;

  result = checkClipboardDataMessages();
  if (result != 0)
    return result;

  result = checkClipboardCancelMessages();
  if (result != 0)
    return result;

  return checkPlatformProperties();
}
