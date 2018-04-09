#pragma once

#include "iobjectinstaller.h"
#include "sessionrecord.h"
#include "libwfp/filterengine.h"
#include <atomic>
#include <memory>
#include <vector>

class SessionController : public IObjectInstaller
{
public:

	SessionController(std::unique_ptr<wfp::FilterEngine> &&engine);
	~SessionController();

	bool addProvider(wfp::ProviderBuilder &providerBuilder) override;
	bool addSublayer(wfp::SublayerBuilder &sublayerBuilder) override;
	bool addFilter(wfp::FilterBuilder &filterBuilder, const wfp::IConditionBuilder &conditionBuilder) override;

	bool executeTransaction(std::function<bool()> operation);
	bool executeReadOnlyTransaction(std::function<bool()> operation);

	//
	// Retrieve checkpoint key that can be used to restore the current session state
	// This should be done outside of an active transaction
	//
	uint32_t checkpoint();

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

	std::unique_ptr<wfp::FilterEngine> m_engine;

	std::vector<SessionRecord> m_records;
	std::vector<SessionRecord> m_transactionRecords;

	std::atomic_bool m_activeTransaction;
};
