/*
 * Copyright (c) 2012-2014 Apple Inc. All rights reserved.
 *
 * @APPLE_OSREFERENCE_LICENSE_HEADER_START@
 *
 * This file contains Original Code and/or Modifications of Original Code
 * as defined in and that are subject to the Apple Public Source License
 * Version 2.0 (the 'License'). You may not use this file except in
 * compliance with the License. The rights granted to you under the License
 * may not be used to create, or enable the creation or redistribution of,
 * unlawful or unlicensed copies of an Apple operating system, or to
 * circumvent, violate, or enable the circumvention or violation of, any
 * terms of an Apple operating system software license agreement.
 *
 * Please obtain a copy of the License at
 * http://www.opensource.apple.com/apsl/ and read it before using this file.
 *
 * The Original Code and all software distributed under the License are
 * distributed on an 'AS IS' basis, WITHOUT WARRANTY OF ANY KIND, EITHER
 * EXPRESS OR IMPLIED, AND APPLE HEREBY DISCLAIMS ALL SUCH WARRANTIES,
 * INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE, QUIET ENJOYMENT OR NON-INFRINGEMENT.
 * Please see the License for the specific language governing rights and
 * limitations under the License.
 *
 * @APPLE_OSREFERENCE_LICENSE_HEADER_END@
 */

#ifndef _NET_PKTAP_H_
#define _NET_PKTAP_H_

#include <sys/_types/_timeval32.h>
#include <stdint.h>
#include <net/if.h>
#include <uuid/uuid.h>

#ifdef PRIVATE

#define PKTAP_IFNAME "pktap"

/* To store interface name + unit */
#define PKTAP_IFXNAMESIZE (IF_NAMESIZE + 8)

/*
 * Commands via SIOCGDRVSPEC/SIOCSDRVSPEC
 */
#define PKTP_CMD_FILTER_GET	1	/* array of PKTAP_MAX_FILTERS * struct pktap_filter */
#define PKTP_CMD_FILTER_SET	3	/* array of PKTAP_MAX_FILTERS * struct pktap_filter */
#define PKTP_CMD_TAP_COUNT	4	/* uint32_t number of active bpf tap on the interface */

/*
 * Filtering is currently based on network interface properties --
 * the interface type and the interface name -- and has two types of 
 * operations -- pass and skip.
 * By default only interfaces of type IFT_ETHER and IFT_CELLULAR pass
 * the filter.
 * It's possible to include other interfaces by type or by name
 * The interface type is evaluated before the interface name
 * The first matching rule stops the evaluation. 
 * A rule with interface type 0 (zero) matches any interfaces
 */
#define PKTAP_FILTER_OP_NONE	0	/* For inactive entries at the end of the list */
#define PKTAP_FILTER_OP_PASS	1
#define PKTAP_FILTER_OP_SKIP	2

#define PKTAP_FILTER_PARAM_NONE		0
#define PKTAP_FILTER_PARAM_IF_TYPE	1
#define PKTAP_FILTER_PARAM_IF_NAME	2

#ifdef BSD_KERNEL_PRIVATE
struct pktap_filter {
	uint32_t	filter_op;
	uint32_t	filter_param;
	union {
		uint32_t	_filter_if_type;
		char		_filter_if_name[PKTAP_IFXNAMESIZE];
	} param_;
	size_t		filter_ifname_prefix_len;
};

struct x_pktap_filter {
#else
struct pktap_filter {
#endif /* BSD_KERNEL_PRIVATE */
	uint32_t	filter_op;
	uint32_t	filter_param;
	union {
		uint32_t	_filter_if_type;
		char		_filter_if_name[PKTAP_IFXNAMESIZE];
	} param_;
};
#define filter_param_if_type param_._filter_if_type
#define filter_param_if_name param_._filter_if_name

#define PKTAP_MAX_FILTERS 8

/*
 * Header for DLT_PKTAP
 *
 * In theory, there could be several types of blocks in a chain before the actual packet
 */
struct pktap_header {
	uint32_t		pth_length;			/* length of this header */
	uint32_t		pth_type_next;			/* type of data following */
	uint32_t		pth_dlt;			/* DLT of packet */
	char			pth_ifname[PKTAP_IFXNAMESIZE];	/* interface name */
	uint32_t		pth_flags;			/* flags */
	uint32_t		pth_protocol_family;
	uint32_t		pth_frame_pre_length;
	uint32_t		pth_frame_post_length;
	pid_t			pth_pid;			/* process ID */
	char			pth_comm[MAXCOMLEN+1];		/* process name */
	uint32_t		pth_svc;			/* service class */
	uint16_t		pth_iftype;
	uint16_t		pth_ifunit;
	pid_t			pth_epid;		/* effective process ID */
	char			pth_ecomm[MAXCOMLEN+1];	/* effective command name */
	uint32_t		pth_flowid;
	uint32_t		pth_ipproto;
	struct timeval32	pth_tstamp;
	uuid_t			pth_uuid;
	uuid_t			pth_euuid;	
};

/*
 *
 */
#define PTH_TYPE_NONE	0		/* No more data following */
#define PTH_TYPE_PACKET	1		/* Actual captured packet data */

#define PTH_FLAG_DIR_IN			0x0001	/* Outgoing packet */
#define PTH_FLAG_DIR_OUT		0x0002	/* Incoming packet */
#define PTH_FLAG_PROC_DELEGATED		0x0004	/* Process delegated */
#define PTH_FLAG_IF_DELEGATED		0x0008	/* Interface delegated */
#ifdef BSD_KERNEL_PRIVATE
#define PTH_FLAG_DELAY_PKTAP		0x1000	/* Finalize pktap header on read */
#endif /* BSD_KERNEL_PRIVATE */
#define PTH_FLAG_TSTAMP			0x2000	/* Has time stamp */
#define	PTH_FLAG_NEW_FLOW		0x4000	/* Packet from a new flow */


#ifdef BSD_KERNEL_PRIVATE
extern void pktap_init(void);
extern void pktap_input(struct ifnet *, protocol_family_t, struct mbuf *, char *);
extern void pktap_output(struct ifnet *, protocol_family_t, struct mbuf *, 
	u_int32_t, u_int32_t);
extern void pktap_fill_proc_info(struct pktap_header *, protocol_family_t , 
	struct mbuf *, u_int32_t , int , struct ifnet *);
extern void pktap_finalize_proc_info(struct pktap_header *);

#endif /* BSD_KERNEL_PRIVATE */
#endif /* PRIVATE */

#endif /* _NET_PKTAP_H_ */
