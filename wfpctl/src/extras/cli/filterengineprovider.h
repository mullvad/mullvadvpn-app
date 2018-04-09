#pragma once

#include "libwfp/filterengine.h"
#include <memory>

class FilterEngineProvider
{
	FilterEngineProvider()
	{
	}

public:

	static FilterEngineProvider &Instance()
	{
		static FilterEngineProvider provider;
		return provider;
	}

	std::shared_ptr<wfp::FilterEngine> get()
	{
		return m_engine;
	}

	//
	// naive set(), good for now
	//
	void set(std::shared_ptr<wfp::FilterEngine> engine)
	{
		m_engine = engine;
	}

private:

	FilterEngineProvider(const FilterEngineProvider &);
	FilterEngineProvider &operator=(const FilterEngineProvider &);

	std::shared_ptr<wfp::FilterEngine> m_engine;
};
