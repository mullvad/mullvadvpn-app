#pragma once

#include <cstdint>
#include <windows.h>

#pragma pack(push, 1)
typedef struct ICON_STREAMS_HEADER_
{
	uint32_t HeaderSize;
	uint32_t u1;					// 7
	uint16_t u2;					// 1
	uint16_t u3;					// 1
	uint32_t NumberRecords;
	uint32_t OffsetFirstRecord;
}
ICON_STREAMS_HEADER;
#pragma pack(pop)

enum ICON_STREAMS_VISIBILITY
{
	NOTIFICATIONS_ONLY = 0,
	HIDE_ICON_AND_NOTIFICATIONS = 1,
	SHOW_ICON_AND_NOTIFICATIONS = 2,
};

#pragma pack(push, 1)
typedef struct ICON_STREAMS_RECORD_
{
	uint16_t ApplicationPath[MAX_PATH];
	uint32_t u1;					// id that the owning application uses to identify the icon?
									// this value is constant except when used for the networking icon
									// where it cycles through a set of ids.

	uint32_t u2;					// 0
	uint32_t Visibility;
	uint16_t YearCreated;
	uint16_t MonthCreated;
	uint16_t LastTooltip[MAX_PATH];
	uint32_t u6;					// 0
	uint32_t u7;					// 0 or 1 but why?
	uint32_t ImagelistId;			// id of cached icon, or -1
	GUID Guid;						//
	uint32_t u8;					// 0
	uint32_t u9;					// 0
	uint32_t u10;					// 0
	FILETIME Time1;					// discrete event 1 UTC
	FILETIME Time2;					// discrete event 2 UTC, this can be 0
	uint32_t u11;					// 0

	union
	{
		struct
		{
			uint16_t ApplicationName[256 + 1];
			uint8_t Padding[6];
		} details;
		struct
		{
			uint32_t u12;			// 200d0000
			uint16_t u13;			// b0fe
			uint16_t ApplicationName[256 + 1];
		} extended_details;
	} DUMMYUNIONNAME;

	uint32_t Ordinal;				// determines ordering within group.
}
ICON_STREAMS_RECORD;
#pragma pack(pop)
