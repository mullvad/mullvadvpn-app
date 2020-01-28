#include "stdafx.h"
#include "sessionrecord.h"
#include "libwfp/objectdeleter.h"
#include <libcommon/error.h>
#include <atomic>
#include <cstdint>

namespace
{

std::atomic<uint32_t> g_keybase = 0;

} // anonymous namespace

SessionRecord::SessionRecord(const GUID &id, WfpObjectType type)
	: m_type(type)
	, m_id(id)
	, m_key(g_keybase++)
{
}

SessionRecord::SessionRecord(UINT64 id)
	: m_type(WfpObjectType::Filter)
	, m_filterId(id)
	, m_key(g_keybase++)
{
}

void SessionRecord::purge(wfp::FilterEngine &engine)
{
	switch (m_type)
	{
		case WfpObjectType::Provider:
		{
			wfp::ObjectDeleter::DeleteProvider(engine, m_id);
			break;
		}
		case WfpObjectType::Sublayer:
		{
			wfp::ObjectDeleter::DeleteSublayer(engine, m_id);
			break;
		}
		case WfpObjectType::Filter:
		{
			wfp::ObjectDeleter::DeleteFilter(engine, m_filterId);
			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

uint32_t SessionRecord::key() const
{
	return m_key;
}
