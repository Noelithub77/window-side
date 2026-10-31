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

#ifndef LGPROTOCOL_KVMFR_TYPES_H
#define LGPROTOCOL_KVMFR_TYPES_H

#include <stdint.h>

typedef enum KVMFRFrameType
{
  FRAME_TYPE_INVALID = 0,
  FRAME_TYPE_BGRA    = 1,
  FRAME_TYPE_RGBA    = 2,
  FRAME_TYPE_RGBA10  = 3,
  FRAME_TYPE_RGBA16F = 4,
  FRAME_TYPE_BGR_32  = 5,
  FRAME_TYPE_RGB_24  = 6,
  FRAME_TYPE_MAX     = 7
}
KVMFRFrameType;

typedef enum KVMFRFrameRotation
{
  FRAME_ROT_0   = 0,
  FRAME_ROT_90  = 1,
  FRAME_ROT_180 = 2,
  FRAME_ROT_270 = 3
}
KVMFRFrameRotation;

typedef enum KVMFRCursorType
{
  CURSOR_TYPE_COLOR        = 0,
  CURSOR_TYPE_MONOCHROME   = 1,
  CURSOR_TYPE_MASKED_COLOR = 2
}
KVMFRCursorType;

typedef uint32_t KVMFRColorTransformFlags;

enum
{
  LG_COLOR_TRANSFORM_MATRIX = 1,
  LG_COLOR_TRANSFORM_LUT    = 2
};

#define LG_SDR_WHITE_LEVEL_DEFAULT 203
#define LG_MAX_FRAME_DAMAGE_RECTS  64

typedef struct KVMFRFrameDamageRect
{
  uint32_t x;
  uint32_t y;
  uint32_t width;
  uint32_t height;
}
KVMFRFrameDamageRect;

typedef struct KVMFRColorTransform
{
  KVMFRColorTransformFlags flags;
  float                    matrix[3][4];
  float                    scalar;
  float                    lut[4096][4];
}
KVMFRColorTransform;

#endif
