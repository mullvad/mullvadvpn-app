#include "stdafx.h"
#include "objectpurger.h"
#include "mullvadguids.h"
#include "wfpobjecttype.h"
#include "libwfp/filterengine.h"
#include "libwfp/objectdeleter.h"
#include "libwfp/transaction.h"
#include "libwfp/objectenumerator.h"
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

template<typename T>
bool HasMullvadProvider(T obj)
{
	return nullptr != obj.providerKey
		&& 0 == memcmp(obj.providerKey, &MullvadGuids::Provider(), sizeof(*obj.providerKey));
}

template<typename T>
bool HasPersistentMullvadProvider(const T &obj)
{
	return nullptr != obj.providerKey
		&& 0 == memcmp(obj.providerKey, &MullvadGuids::ProviderPersistent(), sizeof(*obj.providerKey));
}

} // anonymous namespace

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveFiltersFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		std::vector<GUID> filtersToRemove;
		wfp::ObjectEnumerator::Filters(engine, [&](const auto &filter) -> bool
		{
			// Delete both non-persistent and persistent filters
			if (HasMullvadProvider(filter) || HasPersistentMullvadProvider(filter))
			{
				filtersToRemove.push_back(filter.filterKey);
			}
			return true;
		});

		std::for_each(filtersToRemove.begin(), filtersToRemove.end(), [&](GUID &filterKey) {
			wfp::ObjectDeleter::DeleteFilter(engine, filterKey);
		});
	};
}



//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveAllFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		std::vector<GUID> filtersToRemove;
		wfp::ObjectEnumerator::Filters(engine, [&](const auto &filter) -> bool
		{
			// Delete both non-persistent and persistent filters
			if (HasMullvadProvider(filter) || HasPersistentMullvadProvider(filter))
			{
				filtersToRemove.push_back(filter.filterKey);
			}
			return true;
		});

		std::vector<GUID> sublayersToRemove;
		wfp::ObjectEnumerator::Sublayers(engine, [&](const auto &sublayer) -> bool
		{
			// Delete both non-persistent and persistent sublayers
			if (HasMullvadProvider(sublayer) || HasPersistentMullvadProvider(sublayer))
			{
				sublayersToRemove.push_back(sublayer.subLayerKey);
			}
			return true;
		});

		std::for_each(filtersToRemove.begin(), filtersToRemove.end(), [&](GUID &filterKey) {
			wfp::ObjectDeleter::DeleteFilter(engine, filterKey);
		});

		std::for_each(sublayersToRemove.begin(), sublayersToRemove.end(), [&](GUID &sublayerKey) {
			wfp::ObjectDeleter::DeleteSublayer(engine, sublayerKey);
		});

		wfp::ObjectDeleter::DeleteProvider(engine, MullvadGuids::Provider());
		wfp::ObjectDeleter::DeleteProvider(engine, MullvadGuids::ProviderPersistent());
	};
}

//static
ObjectPurger::RemovalFunctor ObjectPurger::GetRemoveNonPersistentFunctor()
{
	return [](wfp::FilterEngine &engine)
	{
		std::vector<GUID> filtersToRemove;
		wfp::ObjectEnumerator::Filters(engine, [&](const auto &filter) -> bool
		{
			// Delete only non-persistent filters
			if (HasMullvadProvider(filter))
			{
				filtersToRemove.push_back(filter.filterKey);
			}
			return true;
		});

		std::vector<GUID> sublayersToRemove;
		wfp::ObjectEnumerator::Sublayers(engine, [&](const auto &sublayer) -> bool
		{
			// Delete only non-persistent sublayers
			if (HasMullvadProvider(sublayer))
			{
				sublayersToRemove.push_back(sublayer.subLayerKey);
			}
			return true;
		});

		std::for_each(filtersToRemove.begin(), filtersToRemove.end(), [&](GUID &filterKey) {
			wfp::ObjectDeleter::DeleteFilter(engine, filterKey);
		});

		std::for_each(sublayersToRemove.begin(), sublayersToRemove.end(), [&](GUID &sublayerKey) {
			wfp::ObjectDeleter::DeleteSublayer(engine, sublayerKey);
		});

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
