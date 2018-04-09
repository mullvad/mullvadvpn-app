#include "stdafx.h"
#include "sessionrecord.h"
#include "libwfp/objectdeleter.h"
#include <atomic>
#include <cstdint>
#include <stdexcept>

namespace
{

std::atomic<uint32_t> g_keybase = 0;

} // anonymous namespace

SessionRecord::SessionRecord(const GUID &id, ObjectType type)
	: m_type(type)
	, m_id(id)
	, m_key(g_keybase++)
{
}

SessionRecord::SessionRecord(UINT64 id)
	: m_type(ObjectType::Filter)
	, m_filterId(id)
	, m_key(g_keybase++)
{
}

void SessionRecord::purge(wfp::FilterEngine &engine)
{
	switch (m_type)
	{
		case ObjectType::Provider:
		{
			wfp::ObjectDeleter::DeleteProvider(engine, m_id);
			break;
		}
		case ObjectType::Sublayer:
		{
			wfp::ObjectDeleter::DeleteSublayer(engine, m_id);
			break;
		}
		case ObjectType::Filter:
		{
			wfp::ObjectDeleter::DeleteFilter(engine, m_filterId);
			break;
		}
		default:
		{
			throw std::logic_error("Missing case handler in switch clause");
		}
	};
}

uint32_t SessionRecord::key() const
{
	return m_key;
}
