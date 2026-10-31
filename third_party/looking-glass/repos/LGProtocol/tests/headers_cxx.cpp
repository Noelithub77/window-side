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

#if LGPROTOCOL_HEADER_ID == 1
#include <LGProtocol/KVMFR.h>
#elif LGPROTOCOL_HEADER_ID == 2
#include <LGProtocol/KVMFRClipboard.h>
#elif LGPROTOCOL_HEADER_ID == 3
#include <LGProtocol/KVMFRInput.h>
#elif LGPROTOCOL_HEADER_ID == 4
#include <LGProtocol/KVMFRRecovery.h>
#elif LGPROTOCOL_HEADER_ID == 5
#include <LGProtocol/KVMFRStream.h>
#elif LGPROTOCOL_HEADER_ID == 6
#include <LGProtocol/LGMPConfig.h>
#elif LGPROTOCOL_HEADER_ID == 7
#include <LGProtocol/KVMFRTypes.h>
#else
#error Unsupported common-header selector
#endif

int main()
{
  return 0;
}
