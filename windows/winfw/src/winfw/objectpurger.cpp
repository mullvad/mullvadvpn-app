#include "stdafx.h"
#include "objectpurger.h"
#include "mullvadguids.h"
#include "wfpobjecttype.h"
#include "libwfp/filterengine.h"
#include "libwfp/objectdeleter.h"
#include "libwfp/transaction.h"
#include <algorithm>

namespace
{

using ObjectDeleter = std::function<void(wfp::FilterEngine &, const GUID &)>;

template<typename TRange>
void RemoveRange(wfp::FilterEngine &engine, ObjectDeleter deleter, TRange range)
{
	std::for_each(range.first, range.second, [&](const auto &record)
	{
		const GUID &objectId = record.second;
		deleter(engine, objectId);
	});
}

} // anonymous namespace

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveFiltersFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		const auto registry = MullvadGuids::DetailedRegistry(MullvadGuids::IdentityQualifier::IncludeAll);

		// Resolve correct overload.
		void (*deleter)(wfp::FilterEngine &, const GUID &) = wfp::ObjectDeleter::DeleteFilter;

		RemoveRange(engine, deleter, registry.equal_range(WfpObjectType::Filter));
	};
}

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveAllFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		const auto registry = MullvadGuids::DetailedRegistry(MullvadGuids::IdentityQualifier::IncludeAll);

		// Resolve correct overload.
		void(*deleter)(wfp::FilterEngine &, const GUID &) = wfp::ObjectDeleter::DeleteFilter;

		RemoveRange(engine, deleter, registry.equal_range(WfpObjectType::Filter));
		RemoveRange(engine, wfp::ObjectDeleter::DeleteSublayer, registry.equal_range(WfpObjectType::Sublayer));
		RemoveRange(engine, wfp::ObjectDeleter::DeleteProvider, registry.equal_range(WfpObjectType::Provider));
	};
}

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveNonPersistentFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		const auto registry = MullvadGuids::DetailedRegistry(MullvadGuids::IdentityQualifier::IncludeDeprecated);

		// Resolve correct overload.
		void(*deleter)(wfp::FilterEngine &, const GUID &) = wfp::ObjectDeleter::DeleteFilter;

		RemoveRange(engine, deleter, registry.equal_range(WfpObjectType::Filter));
		RemoveRange(engine, wfp::ObjectDeleter::DeleteSublayer, registry.equal_range(WfpObjectType::Sublayer));
		RemoveRange(engine, wfp::ObjectDeleter::DeleteProvider, registry.equal_range(WfpObjectType::Provider));
	};
}

//static
bool ObjectPurger::Execute(RemovalFunctor f)
{
	auto engine = wfp::FilterEngine::StandardSession();

	auto wrapper = [&]()
	{
		return f(*engine), true;
	};

	return wfp::Transaction::Execute(*engine, wrapper);
}
