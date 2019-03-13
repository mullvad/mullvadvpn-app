#pragma once

#include "libwfp/filterengine.h"
#include "wfpobjecttype.h"
#include <guiddef.h>
#include <windows.h>

class SessionRecord
{
public:

	SessionRecord(const GUID &id, WfpObjectType type);
	SessionRecord(UINT64 id);

	SessionRecord(const SessionRecord &) = default;
	SessionRecord(SessionRecord &&) = default;
	SessionRecord &operator=(const SessionRecord &) = default;

	void purge(wfp::FilterEngine &engine);

	uint32_t key() const;

private:

	WfpObjectType m_type;

	GUID m_id;
	UINT64 m_filterId;

	uint32_t m_key;
};
