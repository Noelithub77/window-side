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

#ifndef LGPROTOCOL_KVMFR_CLIPBOARD_H
#define LGPROTOCOL_KVMFR_CLIPBOARD_H

#include "KVMFRStream.h"

#define KVMFR_CLIPBOARD_VERSION                       4
#define KVMFR_CLIPBOARD_STREAM_VERSION                1
#define KVMFR_CLIPBOARD_STREAM_SLOT_COUNT             4
#define KVMFR_CLIPBOARD_STREAM_SLOT_BYTES             262144u
#define KVMFR_CLIPBOARD_STREAM_WINDOW_BYTES           1048576u
#define KVMFR_CLIPBOARD_SLOT_COUNT                    4
#define KVMFR_CLIPBOARD_DATA_BYTES                    262144u
#define KVMFR_CLIPBOARD_REPRESENTATION_BYTES          65536u
#define KVMFR_CLIPBOARD_SIZE_UNKNOWN                  UINT64_MAX
#define KVMFR_CLIPBOARD_FILE_READ_BYTES               1048576u
#define KVMFR_CLIPBOARD_FILE_ROOT_NODE                UINT64_C(0)
#define KVMFR_CLIPBOARD_FILE_MAX_ACQUISITIONS         8
#define KVMFR_CLIPBOARD_FILE_MAX_REQUESTS             32
#define KVMFR_CLIPBOARD_TRANSFER_HELPER               (UINT64_C(1) << 63)
#define KVMFR_CLIPBOARD_FILE_ENTRY_ALIGN              8u
#define KVMFR_CLIPBOARD_STREAM_RECORD_BYTES           262208u
#define KVMFR_CLIPBOARD_FILE_ENTRY_BYTES(nameLength)  \
  ((size_t)40u + (((size_t)(nameLength) + (size_t)7u) & ~(size_t)7u))

#define KVMFR_CLIPBOARD_QUEUE_UDATA(type, serial)     \
  (((uint64_t)(uint32_t)(type) << 32u) | (uint64_t)(uint32_t)(serial))
#define KVMFR_CLIPBOARD_QUEUE_TYPE(udata)             \
  ((KVMFRClipboardQueueType)((uint64_t)(udata) >> 32u))
#define KVMFR_CLIPBOARD_QUEUE_SERIAL(udata)           \
  ((uint32_t)(uint64_t)(udata))

typedef uint32_t KVMFRClipboardFormat;

enum
{
  KVMFR_CLIPBOARD_FORMAT_NONE  = 0,
  KVMFR_CLIPBOARD_FORMAT_TEXT  = 1,
  KVMFR_CLIPBOARD_FORMAT_PNG   = 2,
  KVMFR_CLIPBOARD_FORMAT_BMP   = 3,
  KVMFR_CLIPBOARD_FORMAT_TIFF  = 4,
  KVMFR_CLIPBOARD_FORMAT_JPEG  = 5,
  KVMFR_CLIPBOARD_FORMAT_FILES = 6
};

typedef uint32_t KVMFRClipboardFormatFlags;

enum
{
  KVMFR_CLIPBOARD_FORMAT_MASK_TEXT  = 1,
  KVMFR_CLIPBOARD_FORMAT_MASK_PNG   = 2,
  KVMFR_CLIPBOARD_FORMAT_MASK_BMP   = 4,
  KVMFR_CLIPBOARD_FORMAT_MASK_TIFF  = 8,
  KVMFR_CLIPBOARD_FORMAT_MASK_JPEG  = 16,
  KVMFR_CLIPBOARD_FORMAT_MASK_FILES = 32,
  KVMFR_CLIPBOARD_FORMAT_MASK_ALL   = 63
};

typedef uint32_t KVMFRClipboardMessageType;

enum
{
  KVMFR_CLIPBOARD_MESSAGE_CLAIM         = 1,
  KVMFR_CLIPBOARD_MESSAGE_RELEASE       = 2,
  KVMFR_CLIPBOARD_MESSAGE_KEEPALIVE     = 3,
  KVMFR_CLIPBOARD_MESSAGE_OFFER         = 4,
  KVMFR_CLIPBOARD_MESSAGE_CLEAR         = 5,
  KVMFR_CLIPBOARD_MESSAGE_REQUEST       = 6,
  KVMFR_CLIPBOARD_MESSAGE_DATA          = 7,
  KVMFR_CLIPBOARD_MESSAGE_CANCEL        = 8,
  KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE  = 9,
  KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRED = 10,
  KVMFR_CLIPBOARD_MESSAGE_FILE_RELEASE  = 11,
  KVMFR_CLIPBOARD_MESSAGE_FILE_REQUEST  = 12,
  KVMFR_CLIPBOARD_MESSAGE_FILE_DATA     = 13,
  KVMFR_CLIPBOARD_MESSAGE_FILE_CANCEL   = 14
};

typedef uint32_t KVMFRClipboardFlags;

enum
{
  KVMFR_CLIPBOARD_FLAG_BEGIN = 1,
  KVMFR_CLIPBOARD_FLAG_END   = 2
};

typedef uint32_t KVMFRClipboardFileOperation;

enum
{
  KVMFR_CLIPBOARD_FILE_OP_LIST = 1,
  KVMFR_CLIPBOARD_FILE_OP_READ = 2
};

typedef uint32_t KVMFRClipboardFileType;

enum
{
  KVMFR_CLIPBOARD_FILE_TYPE_REGULAR   = 1,
  KVMFR_CLIPBOARD_FILE_TYPE_DIRECTORY = 2
};

typedef uint32_t KVMFRClipboardFileError;

enum
{
  KVMFR_CLIPBOARD_FILE_ERROR_NONE                = 0,
  KVMFR_CLIPBOARD_FILE_ERROR_NOT_FOUND           = 1,
  KVMFR_CLIPBOARD_FILE_ERROR_ACCESS              = 2,
  KVMFR_CLIPBOARD_FILE_ERROR_NOT_DIRECTORY       = 3,
  KVMFR_CLIPBOARD_FILE_ERROR_IS_DIRECTORY        = 4,
  KVMFR_CLIPBOARD_FILE_ERROR_IO                  = 5,
  KVMFR_CLIPBOARD_FILE_ERROR_INVALID             = 6,
  KVMFR_CLIPBOARD_FILE_ERROR_NO_MEMORY           = 7,
  KVMFR_CLIPBOARD_FILE_ERROR_NO_SPACE            = 8,
  KVMFR_CLIPBOARD_FILE_ERROR_DISCONNECTED        = 9,
  KVMFR_CLIPBOARD_FILE_ERROR_CANCELLED           = 10,
  KVMFR_CLIPBOARD_FILE_ERROR_NOT_SUPPORTED       = 11,
  KVMFR_CLIPBOARD_FILE_ERROR_STALE               = 12
};

typedef uint32_t KVMFRClipboardStatusFlags;

enum
{
  KVMFR_CLIPBOARD_STATUS_AVAILABLE = 1,
  KVMFR_CLIPBOARD_STATUS_HAS_OWNER = 2
};

typedef uint32_t KVMFRClipboardQueueType;

enum
{
  KVMFR_CLIPBOARD_QUEUE_STATUS  = 1,
  KVMFR_CLIPBOARD_QUEUE_MESSAGE = 2
};

typedef struct KVMFRClipboardMessage
{
  uint32_t                  version;
  KVMFRClipboardMessageType type;
  uint32_t                  generation;
  uint32_t                  sequence;
  uint64_t                  clipboardGeneration;
  uint64_t                  transfer;
  uint64_t                  offset;
  uint64_t                  size;
  KVMFRClipboardFormat      format;
  uint32_t                  flags;
  uint32_t                  token;
  uint32_t                  length;
}
KVMFRClipboardMessage;

typedef struct KVMFRClipboardFileEntry
{
  uint64_t               node;
  uint64_t               size;
  uint64_t               createdNs;
  uint64_t               modifiedNs;
  KVMFRClipboardFileType type;
  uint32_t               nameLength;
}
KVMFRClipboardFileEntry;

typedef struct KVMFRClipboardStatus
{
  uint32_t                     version;
  KVMFRClipboardStatusFlags    flags;
  uint32_t                     generation;
  uint32_t                     ownerClientID;
  uint32_t                     ownerGeneration;
  uint32_t                     lease;
  KVMFRClipboardFormatFlags    formats;
  uint32_t                     slotBytes;
  uint32_t                     streamVersion;
  uint32_t                     streamSlotCount;
  uint32_t                     reserved[6];
  KVMFRStreamDescriptor        hostToClient;
  KVMFRStreamDescriptor        clientToHost;
}
KVMFRClipboardStatus;

typedef KVMFRClipboardMessage KVMFRClipboardSlotHeader;

static inline int kvmfrClipboardTransferFromHelper(uint64_t transfer)
{
  return (transfer & KVMFR_CLIPBOARD_TRANSFER_HELPER) != 0;
}

static inline int kvmfrClipboardTransferFromClient(uint64_t transfer)
{
  return transfer != 0 && !kvmfrClipboardTransferFromHelper(transfer);
}

static inline int kvmfrClipboardFormatValid(KVMFRClipboardFormat format)
{
  return format >= KVMFR_CLIPBOARD_FORMAT_TEXT &&
    format <= KVMFR_CLIPBOARD_FORMAT_FILES;
}

static inline int kvmfrClipboardRepresentationFormatValid(
  KVMFRClipboardFormat format)
{
  return format >= KVMFR_CLIPBOARD_FORMAT_TEXT &&
    format <= KVMFR_CLIPBOARD_FORMAT_JPEG;
}

static inline KVMFRClipboardFormatFlags kvmfrClipboardFormatFlag(
  KVMFRClipboardFormat format)
{
  if (!kvmfrClipboardFormatValid(format))
    return 0;

  return (KVMFRClipboardFormatFlags)(UINT32_C(1) << (format - 1u));
}

static inline int kvmfrClipboardFileErrorValid(
  KVMFRClipboardFileError error)
{
  return error <= KVMFR_CLIPBOARD_FILE_ERROR_STALE;
}

static inline int kvmfrClipboardFileOperationValid(
  KVMFRClipboardFileOperation operation)
{
  return operation >= KVMFR_CLIPBOARD_FILE_OP_LIST &&
    operation <= KVMFR_CLIPBOARD_FILE_OP_READ;
}

static inline int kvmfrClipboardFileTransferValid(uint64_t transfer)
{
  return transfer != 0;
}

static inline int kvmfrClipboardFileMessageType(
  KVMFRClipboardMessageType type)
{
  return type >= KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE &&
    type <= KVMFR_CLIPBOARD_MESSAGE_FILE_CANCEL;
}

static inline int kvmfrClipboardFileMessageValid(
  const KVMFRClipboardMessage * message)
{
  KVMFRClipboardFileOperation operation;
  uint64_t                    endOffset;

  if (!message ||
      message->version != KVMFR_CLIPBOARD_VERSION ||
      !kvmfrClipboardFileMessageType(message->type) ||
      message->clipboardGeneration == 0 ||
      !kvmfrClipboardFileTransferValid(message->transfer) ||
      message->format != KVMFR_CLIPBOARD_FORMAT_FILES ||
      message->length > KVMFR_CLIPBOARD_DATA_BYTES)
    return 0;

  switch (message->type)
  {
    case KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRE:
    case KVMFR_CLIPBOARD_MESSAGE_FILE_RELEASE:
      return message->sequence == 0 &&
        message->offset == 0 &&
        message->size == 0 &&
        message->flags == 0 &&
        message->token == 0 &&
        message->length == 0;

    case KVMFR_CLIPBOARD_MESSAGE_FILE_ACQUIRED:
      return message->sequence == 0 &&
        message->offset == 0 &&
        message->size == 0 &&
        message->flags == 0 &&
        kvmfrClipboardFileErrorValid(message->token) &&
        message->length == 0;

    case KVMFR_CLIPBOARD_MESSAGE_FILE_REQUEST:
      if (message->sequence != 0 || message->length != 0)
        return 0;

      operation = (KVMFRClipboardFileOperation)message->token;
      if (!kvmfrClipboardFileOperationValid(operation))
        return 0;

      if (operation == KVMFR_CLIPBOARD_FILE_OP_LIST)
        return message->offset == 0 && message->flags == 0;

      return message->size != KVMFR_CLIPBOARD_FILE_ROOT_NODE &&
        message->flags >= 1 &&
        message->flags <= KVMFR_CLIPBOARD_FILE_READ_BYTES &&
        message->offset <= UINT64_MAX - ((uint64_t)message->flags - 1u);

    case KVMFR_CLIPBOARD_MESSAGE_FILE_DATA:
      operation = (KVMFRClipboardFileOperation)message->token;
      if (!kvmfrClipboardFileOperationValid(operation) ||
          (message->flags & ~(uint32_t)(KVMFR_CLIPBOARD_FLAG_BEGIN |
            KVMFR_CLIPBOARD_FLAG_END)) != 0 ||
          (message->length == 0 &&
            (message->flags & KVMFR_CLIPBOARD_FLAG_END) == 0) ||
          message->offset > UINT64_MAX - message->length)
        return 0;

      endOffset = message->offset + message->length;

      if ((message->flags & KVMFR_CLIPBOARD_FLAG_END) != 0)
        return message->size == endOffset;

      if ((message->flags & KVMFR_CLIPBOARD_FLAG_BEGIN) == 0)
        return message->size == KVMFR_CLIPBOARD_SIZE_UNKNOWN;

      return message->size == KVMFR_CLIPBOARD_SIZE_UNKNOWN ||
        message->size >= endOffset;

    case KVMFR_CLIPBOARD_MESSAGE_FILE_CANCEL:
      return message->sequence == 0 &&
        message->offset == 0 &&
        message->size == 0 &&
        message->flags == 0 &&
        message->token != KVMFR_CLIPBOARD_FILE_ERROR_NONE &&
        kvmfrClipboardFileErrorValid(message->token) &&
        message->length == 0;

    default:
      return 0;
  }
}

#if defined(__cplusplus) && __cplusplus >= 201103L
#define LGPROTOCOL_CLIPBOARD_ASSERT(condition, message) static_assert(condition, message)
#elif defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
#define LGPROTOCOL_CLIPBOARD_ASSERT(condition, message) _Static_assert(condition, message)
#endif

#if defined(LGPROTOCOL_CLIPBOARD_ASSERT)
LGPROTOCOL_CLIPBOARD_ASSERT(sizeof(KVMFRClipboardMessage) == 64,
  "KVMFRClipboardMessage size changed");
LGPROTOCOL_CLIPBOARD_ASSERT(sizeof(KVMFRClipboardFileEntry) == 40,
  "KVMFRClipboardFileEntry size changed");
LGPROTOCOL_CLIPBOARD_ASSERT(offsetof(KVMFRClipboardStatus, streamVersion) == 32,
  "KVMFRClipboardStatus.streamVersion offset changed");
LGPROTOCOL_CLIPBOARD_ASSERT(offsetof(KVMFRClipboardStatus, hostToClient) == 64,
  "KVMFRClipboardStatus.hostToClient offset changed");
LGPROTOCOL_CLIPBOARD_ASSERT(sizeof(KVMFRClipboardStatus) == 128,
  "KVMFRClipboardStatus size changed");
LGPROTOCOL_CLIPBOARD_ASSERT(sizeof(KVMFRClipboardSlotHeader) == 64,
  "KVMFRClipboardSlotHeader size changed");
LGPROTOCOL_CLIPBOARD_ASSERT(
  sizeof(KVMFRClipboardMessage) <= KVMFR_CLIPBOARD_REPRESENTATION_BYTES,
  "KVMFRClipboardMessage exceeds its control area");
LGPROTOCOL_CLIPBOARD_ASSERT(
  sizeof(KVMFRClipboardSlotHeader) + KVMFR_CLIPBOARD_DATA_BYTES <=
    KVMFR_CLIPBOARD_STREAM_RECORD_BYTES,
  "KVMFRClipboard stream record is too small");
#undef LGPROTOCOL_CLIPBOARD_ASSERT
#endif

#endif
