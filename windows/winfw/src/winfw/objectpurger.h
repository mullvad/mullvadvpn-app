#pragma once

#include "winfw.h"
#include "libwfp/filterengine.h"
#include <cstdint>
#include <functional>

class ObjectPurger
{
public:

	ObjectPurger() = delete;

	using RemovalFunctor = std::function<void(wfp::FilterEngine &engine)>;

	static RemovalFunctor GetRemoveFiltersFunctor();
	static RemovalFunctor GetRemoveAllFunctor();
	static RemovalFunctor GetRemoveNonPersistentFunctor();

	static bool Execute(RemovalFunctor f);
};
