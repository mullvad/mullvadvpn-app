#pragma once

#include "libwfp/filterengine.h"
#include <guiddef.h>
#include <windows.h>

class SessionRecord
{
public:

	enum class ObjectType
	{
		Provider,
		Sublayer,
		Filter
	};

	SessionRecord(const GUID &id, ObjectType type);
	SessionRecord(UINT64 id);

	SessionRecord(const SessionRecord &) = default;
	SessionRecord(SessionRecord &&) = default;
	SessionRecord &operator=(const SessionRecord &) = default;

	void purge(wfp::FilterEngine &engine);

	uint32_t key() const;

private:

	ObjectType m_type;

	GUID m_id;
	UINT64 m_filterId;

	uint32_t m_key;
};
