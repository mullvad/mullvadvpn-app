#pragma once

#include "iconnection.h"
#include <string>
#include <vector>
#include <stdexcept>
#include <comutil.h>

namespace wmi
{

class MethodCall
{
public:

	void addArgument(const std::wstring &name, _variant_t value);
	void addNullArgument(const std::wstring &name, CIMTYPE type);

	_variant_t invoke(IConnection &connection, CComPtr<IWbemClassObject> instance, const std::wstring &methodName);

private:

	class Argument
	{
	public:

		Argument(const std::wstring &name, _variant_t value)
			: m_name(name)
			, m_value(value)
		{
			if (VT_NULL == V_VT(&value) || VT_EMPTY == V_VT(&value))
			{
				throw std::runtime_error("Cannot add null-argument without specifying type.");
			}
		}

		Argument(const std::wstring &name, CIMTYPE type)
			: m_name(name)
			, m_type(type)
		{
		}

		bool nullValue() const
		{
			return VT_EMPTY == V_VT(&m_value);
		}

		const std::wstring &name() const
		{
			return m_name;
		}

		const _variant_t &value() const
		{
			return m_value;
		}

		CIMTYPE type() const
		{
			return m_type;
		}

	private:

		std::wstring m_name;
		_variant_t m_value;

		// Explicitly specify type when the value is NULL.
		CIMTYPE m_type;
	};

	std::vector<Argument> m_arguments;
};

}
