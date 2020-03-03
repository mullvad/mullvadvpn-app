#pragma once

#include "iobjectinstaller.h"
#include "sessionrecord.h"
#include "mullvadguids.h"
#include <libwfp/filterengine.h>
#include <libwfp/iidentifiable.h>
#include <functional>
#include <atomic>
#include <memory>
#include <vector>

class SessionController : public IObjectInstaller
{
public:

	SessionController(std::unique_ptr<wfp::FilterEngine> &&engine);

	bool addProvider(wfp::ProviderBuilder &providerBuilder) override;
	bool addSublayer(wfp::SublayerBuilder &sublayerBuilder) override;
	bool addFilter(wfp::FilterBuilder &filterBuilder, const wfp::IConditionBuilder &conditionBuilder) override;

	using TransactionFunctor = std::function<bool(SessionController &, wfp::FilterEngine &)>;

	bool executeTransaction(TransactionFunctor operation);
	bool executeReadOnlyTransaction(TransactionFunctor operation);

	//
	// Retrieve checkpoint key that can be used to restore the current session state
	// This should be done outside of an active transaction
	//
	uint32_t checkpoint();

	//
	// Hack. Read checkpoint while currently inside a transaction.
	//
	uint32_t peekCheckpoint();

	//
	// Purge objects in the stack and return to an earlier state
	// Use only inside active transaction
	//
	void revert(uint32_t key);

	//
	// Purge all objects in the stack
	// Use only inside active transaction
	//
	void reset();

private:

	SessionController(const SessionController &) = delete;
	SessionController &operator=(const SessionController &) = delete;

	void rewindState(size_t steps);
	void validateObject(const wfp::IIdentifiable &object) const;

	std::unique_ptr<wfp::FilterEngine> m_engine;

	// Implement cache here since the source data doesn't change.
	const MullvadGuids::IdentityRegistry m_identityRegistry;

	std::vector<SessionRecord> m_records;
	std::vector<SessionRecord> m_transactionRecords;

	std::atomic_bool m_activeTransaction;
};
