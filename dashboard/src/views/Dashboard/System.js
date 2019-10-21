import React, { Component } from 'react';
import {
    ButtonDropdown,
    ButtonGroup,
    ButtonToolbar,
    Dropdown,
    DropdownItem,
    DropdownMenu,
    DropdownToggle,
    Card,
    CardBody,
    CardHeader,
    Col,
    Progress,
    Row
} from 'reactstrap';


class System extends Component {
    constructor(props) {
        super(props);

        this.state = {
            is_open: false
        }
    }

    onUpdate = () => {
        this.setState({});
    }

    componentDidMount() {
        this.props.system.on('change', this.onUpdate);
    }

    componentWillUnmount() {
        this.props.system.off('change', this.onUpdate);
    }
    render() {
        return (
            <Card className="text-white bg-info">
                <CardHeader className="text-value">
                    {this.props.system.name}
                </CardHeader>
                <CardBody>
                    <div className="text-value">{this.props.system.framerate} FPS</div>
                    <div>{this.props.system.components}</div>
                </CardBody>
            </Card>
        )
    }
}

export default System;
