#include "stdafx.h"
#include "objectpurger.h"
#include "mullvadguids.h"
#include "libwfp/filterengine.h"
#include "libwfp/objectdeleter.h"
#include "libwfp/transaction.h"
#include "libwfp/objectenumerator.h"
#include <set>
#include <algorithm>

namespace
{

using ObjectDeleter = std::function<void(wfp::FilterEngine &, const GUID &)>;

template<typename T>
bool HasMullvadProvider(T obj)
{
	return nullptr != obj.providerKey && *obj.providerKey == MullvadGuids::Provider();
}

template<typename T>
bool HasPersistentMullvadProvider(const T &obj)
{
	return nullptr != obj.providerKey && *obj.providerKey == MullvadGuids::ProviderPersistent();
}

} // anonymous namespace

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveAllFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		std::unordered_set<GUID> filtersToRemove;
		wfp::ObjectEnumerator::Filters(engine, [&](const auto &filter) -> bool
		{
			// Delete both non-persistent and persistent filters
			if (HasMullvadProvider(filter) || HasPersistentMullvadProvider(filter))
			{
				filtersToRemove.insert(filter.filterKey);
			}
			return true;
		});

		std::unordered_set<GUID> sublayersToRemove;
		wfp::ObjectEnumerator::Sublayers(engine, [&](const auto &sublayer) -> bool
		{
			// Delete both non-persistent and persistent sublayers
			if (HasMullvadProvider(sublayer) || HasPersistentMullvadProvider(sublayer))
			{
				sublayersToRemove.insert(sublayer.subLayerKey);
			}
			return true;
		});

		for (const auto &filter : filtersToRemove)
		{
			wfp::ObjectDeleter::DeleteFilter(engine, filter);
		}

		for (const auto &sublayer : sublayersToRemove)
		{
			wfp::ObjectDeleter::DeleteSublayer(engine, sublayer);
		}

		wfp::ObjectDeleter::DeleteProvider(engine, MullvadGuids::Provider());
		wfp::ObjectDeleter::DeleteProvider(engine, MullvadGuids::ProviderPersistent());
	};
}

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveNonPersistentFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		std::unordered_set<GUID> filtersToRemove;
		wfp::ObjectEnumerator::Filters(engine, [&](const auto &filter) -> bool
		{
			// Delete only non-persistent filters
			if (HasMullvadProvider(filter))
			{
				filtersToRemove.insert(filter.filterKey);
			}
			return true;
		});

		std::unordered_set<GUID> sublayersToRemove;
		wfp::ObjectEnumerator::Sublayers(engine, [&](const auto &sublayer) -> bool
		{
			// Delete only non-persistent sublayers
			if (HasMullvadProvider(sublayer))
			{
				sublayersToRemove.insert(sublayer.subLayerKey);
			}
			return true;
		});

		for (const auto &filter : filtersToRemove)
		{
			wfp::ObjectDeleter::DeleteFilter(engine, filter);
		}

		for (const auto &sublayer : sublayersToRemove)
		{
			wfp::ObjectDeleter::DeleteSublayer(engine, sublayer);
		}

		wfp::ObjectDeleter::DeleteProvider(engine, MullvadGuids::Provider());
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
