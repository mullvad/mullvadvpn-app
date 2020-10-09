#include "stdafx.h"
#include "sessioncontroller.h"
#include "wfpobjecttype.h"
#include <libwfp/objectinstaller.h>
#include <libwfp/objectdeleter.h>
#include <libwfp/transaction.h>
#include <libcommon/memory.h>
#include <libcommon/error.h>
#include <utility>

namespace
{

template<typename T>
void EraseBack(T &container, size_t elements)
{
	if (elements >= container.size())
	{
		container.clear();
	}
	else
	{
		container.erase
		(
			std::next(container.begin(), container.size() - elements),
			container.end()
		);
	}
}

template<typename T>
void ProcessReverse(T &container, size_t elements, std::function<void(typename T::value_type &)> f)
{
	auto it = container.rbegin();
	auto end = std::next(it, elements);

	while (it != end)
	{
		f(*it++);
	}
}

bool CheckpointKeyToIndex(const std::vector<SessionRecord> &container, uint32_t key, size_t &elementIndex)
{
	auto index = 0;

	for (auto it = container.begin(); it != container.end(); ++it, ++index)
	{
		if (it->key() == key)
		{
			elementIndex = index;
			return true;
		}
	}

	return false;
}

} // anonymous namespace

SessionController::SessionController(std::unique_ptr<wfp::FilterEngine> &&engine)
	: m_engine(std::move(engine))
	, m_identityRegistry(MullvadGuids::Registry(MullvadGuids::IdentityQualifier::IncludePersistent))
	, m_activeTransaction(false)
{
}

bool SessionController::addProvider(wfp::ProviderBuilder &providerBuilder)
{
	if (false == m_activeTransaction)
	{
		THROW_ERROR("Cannot add provider outside transaction");
	}

	validateObject(providerBuilder);

	GUID key;

	auto status = wfp::ObjectInstaller::AddProvider(*m_engine, providerBuilder, &key);

	if (status)
	{
		m_transactionRecords.emplace_back(SessionRecord(key, WfpObjectType::Provider));
	}

	return status;
}

bool SessionController::addSublayer(wfp::SublayerBuilder &sublayerBuilder)
{
	if (false == m_activeTransaction)
	{
		THROW_ERROR("Cannot add sublayer outside transaction");
	}

	validateObject(sublayerBuilder);

	GUID key;

	auto status = wfp::ObjectInstaller::AddSublayer(*m_engine, sublayerBuilder, &key);

	if (status)
	{
		m_transactionRecords.emplace_back(SessionRecord(key, WfpObjectType::Sublayer));
	}

	return status;
}

bool SessionController::addFilter(wfp::FilterBuilder &filterBuilder, const wfp::IConditionBuilder &conditionBuilder)
{
	if (false == m_activeTransaction)
	{
		THROW_ERROR("Cannot add filter outside transaction");
	}

	validateObject(filterBuilder);

	UINT64 id;

	auto status = wfp::ObjectInstaller::AddFilter(*m_engine, filterBuilder, conditionBuilder, &id);

	if (status)
	{
		m_transactionRecords.emplace_back(SessionRecord(id));
	}

	return status;
}

bool SessionController::executeTransaction(TransactionFunctor operation)
{
	if (m_activeTransaction.exchange(true))
	{
		THROW_ERROR("Recursive/concurrent transactions are not supported");
	}

	common::memory::ScopeDestructor scopeDestructor;

	scopeDestructor += [this]()
	{
		m_activeTransaction.store(false);
	};

	m_transactionRecords = m_records;

	auto transactionForwarder = [this, operation]()
	{
		return operation(*this, *m_engine);
	};

	auto status = wfp::Transaction::Execute(*m_engine, transactionForwarder);

	if (status)
	{
		m_records.swap(m_transactionRecords);
	}

	return status;
}

bool SessionController::executeReadOnlyTransaction(TransactionFunctor operation)
{
	if (m_activeTransaction.exchange(true))
	{
		THROW_ERROR("Recursive/concurrent transactions are not supported");
	}

	common::memory::ScopeDestructor scopeDestructor;

	scopeDestructor += [this]()
	{
		m_activeTransaction.store(false);
	};

	auto transactionForwarder = [this, operation]()
	{
		return operation(*this, *m_engine);
	};

	return wfp::Transaction::ExecuteReadOnly(*m_engine, transactionForwarder);
}

uint32_t SessionController::checkpoint()
{
	if (m_activeTransaction)
	{
		THROW_ERROR("Cannot read checkpoint key while in transaction");
	}

	if (m_records.empty())
	{
		return 0;
	}

	return m_records.back().key();
}

uint32_t SessionController::peekCheckpoint()
{
	if (m_transactionRecords.empty())
	{
		return 0;
	}

	return m_transactionRecords.back().key();
}

void SessionController::revert(uint32_t key)
{
	if (false == m_activeTransaction)
	{
		THROW_ERROR("Cannot revert session state outside transaction");
	}

	size_t elementIndex = 0;

	if (false == CheckpointKeyToIndex(m_transactionRecords, key, elementIndex))
	{
		THROW_ERROR("Invalid checkpoint key (checkpoint may have been overwritten?)");
	}

	const size_t numRemove = m_transactionRecords.size() - (elementIndex + 1);

	rewindState(numRemove);
}

void SessionController::reset()
{
	if (false == m_activeTransaction)
	{
		THROW_ERROR("Cannot reset session state outside transaction");
	}

	rewindState(m_transactionRecords.size());
}

void SessionController::rewindState(size_t steps)
{
	auto purged = 0;

	try
	{
		ProcessReverse(m_transactionRecords, steps, [this, &purged](SessionRecord &record)
		{
			record.purge(*m_engine);
			++purged;
		});
	}
	catch (...)
	{
		EraseBack(m_transactionRecords, purged);
		throw;
	}

	EraseBack(m_transactionRecords, steps);
}

void SessionController::validateObject(const wfp::IIdentifiable &object) const
{
	if (m_identityRegistry.end() == m_identityRegistry.find(object.id()))
	{
		THROW_ERROR("Attempting to install non-registered WFP object");
	}
}
